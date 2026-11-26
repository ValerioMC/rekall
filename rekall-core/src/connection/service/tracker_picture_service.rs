use std::collections::HashMap;
use std::sync::Arc;

use rekall_kernel::{Id, RekallError, Result};
use rekall_store::repository as repo;
use rekall_store::{tracker_picture, ConnectionKind};
use tracing::{info, warn};

use crate::connection::rule::tracker_markdown::{pictures_as_uploads, PictureMarkup};
use crate::connection::rule::tracker_name::tracker_name;
use crate::connection::{TaskTracker, TrackerEndpoint, TrackerFailure};
use crate::picture::{PicturePath, PictureReference, PictureStore};
use crate::{in_read, in_write, Ctx};

/// Uploads the pictures a task's text shows to its task manager, once each per repository or Jira
/// issue, and answers the text showing the manager's copies.
#[derive(Clone)]
pub struct TrackerPictureService {
    ctx: Ctx,
    store: Arc<dyn PictureStore>,
    tracker: Arc<dyn TaskTracker>,
}

impl TrackerPictureService {
    pub fn new(ctx: Ctx, store: Arc<dyn PictureStore>, tracker: Arc<dyn TaskTracker>) -> Self {
        Self { ctx, store, tracker }
    }

    /// Each upload is remembered as soon as the manager takes it, so a push that fails later does
    /// not send it again. `issue_key` is where Jira keeps an attachment.
    pub async fn send(&self, endpoint: &TrackerEndpoint, task_id: Id, issue_key: Option<&str>, markdown: &str) -> Result<String> {
        let references = PictureReference::all_in(markdown);
        if references.is_empty() {
            return Ok(markdown.to_string());
        }
        let markup = PictureMarkup::of(endpoint)?;
        let destination = destination_of(endpoint, issue_key)?;
        let mut uploads = self.remembered(task_id, &destination).await?;
        for PictureReference { picture, .. } in references {
            if uploads.contains_key(picture.relative()) {
                continue;
            }
            if let Some(remote) = self.upload(endpoint, issue_key, &picture).await? {
                self.remember(task_id, &destination, &picture, &remote).await?;
                info!(%task_id, picture = picture.relative(), "A picture was uploaded to the task manager");
                uploads.insert(picture.relative().to_string(), remote);
            }
        }
        Ok(pictures_as_uploads(markdown, markup, &uploads))
    }

    /// None for a picture missing from disk, which reads as its description rather than blocking every push.
    async fn upload(&self, endpoint: &TrackerEndpoint, issue_key: Option<&str>, picture: &PicturePath) -> Result<Option<String>> {
        let bytes = match self.store.read(picture).await {
            Ok(bytes) => bytes,
            Err(RekallError::NotFound(reason)) => {
                warn!(%reason, picture = picture.relative(), "A picture missing from disk is sent as its description");
                return Ok(None);
            }
            Err(failure) => return Err(failure),
        };
        let taken = self.tracker.upload_picture(endpoint, issue_key, picture, bytes).await;
        taken.map(Some).map_err(|failure| RekallError::conflict(refusal_of(endpoint.kind, picture, &failure)))
    }

    async fn remembered(&self, task_id: Id, destination: &str) -> Result<HashMap<String, String>> {
        let uploads = in_read!(&self.ctx, |tx| repo::tracker_picture::list_of_destination(tx.db(), task_id, destination).await)?;
        Ok(uploads.into_iter().map(|upload| (upload.picture_path, upload.remote_reference)).collect())
    }

    async fn remember(&self, task_id: Id, destination: &str, picture: &PicturePath, remote: &str) -> Result<()> {
        let upload = tracker_picture::Model {
            task_id,
            destination: destination.to_string(),
            picture_path: picture.relative().to_string(),
            remote_reference: remote.to_string(),
            created_at: self.ctx.now(),
        };
        in_write!(&self.ctx, |tx| repo::tracker_picture::save(tx.db(), upload).await)
    }
}

/// Where an upload lives: the issue on Jira, which holds its own attachments, else the repository.
fn destination_of(endpoint: &TrackerEndpoint, issue_key: Option<&str>) -> Result<String> {
    let site = endpoint.site_url.trim_end_matches('/');
    match (endpoint.kind, issue_key) {
        (ConnectionKind::Jira, Some(key)) => Ok(format!("{site}/browse/{key}")),
        (ConnectionKind::Jira, None) => Err(RekallError::illegal("A Jira attachment belongs to an issue, and this task has none.")),
        _ => Ok(format!("{site}/{}", endpoint.location)),
    }
}

fn refusal_of(kind: ConnectionKind, picture: &PicturePath, failure: &TrackerFailure) -> String {
    let name = tracker_name(kind);
    match failure {
        TrackerFailure::Rejected { status } => format!("{name} rejected the stored credential (HTTP {status}). Replace it on the Connections page."),
        TrackerFailure::Failed { reason } => format!("{name} could not take the picture {}: {reason}.", picture.file_name()),
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/connection/service/tracker_picture_service_tests.rs"]
mod tests;
