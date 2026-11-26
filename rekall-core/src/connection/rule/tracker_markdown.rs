//! How the text sent to a task manager shows the pictures of a task: as the manager's own copies
//! once uploaded, as their descriptions otherwise. An alignment ending is dropped either way.

use std::collections::HashMap;

use rekall_kernel::{RekallError, Result};
use rekall_store::ConnectionKind;

use crate::connection::{AtlassianCredential, AtlassianDeployment, TrackerEndpoint};
use crate::picture::PictureReference;

/// What a task manager's text writes to show a picture the manager holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PictureMarkup {
    /// GitHub and GitLab: a markdown picture at the address the upload answered.
    MarkdownPicture,
    /// Jira Data Center: the wiki macro naming the attachment.
    AttachmentMacro,
    /// Jira Cloud: a link naming the attachment, since showing one in its document format is not documented.
    AttachmentLink,
}

impl PictureMarkup {
    pub fn of(endpoint: &TrackerEndpoint) -> Result<Self> {
        match endpoint.kind {
            ConnectionKind::Github | ConnectionKind::Gitlab => Ok(Self::MarkdownPicture),
            ConnectionKind::Jira => Ok(match AtlassianCredential::from_stored(&endpoint.credential)?.deployment() {
                AtlassianDeployment::Cloud => Self::AttachmentLink,
                AtlassianDeployment::DataCenter => Self::AttachmentMacro,
            }),
            ConnectionKind::Kubernetes | ConnectionKind::Grafana | ConnectionKind::Confluence => {
                Err(RekallError::illegal("Only a GitLab, GitHub or Jira connection takes pictures."))
            }
        }
    }

    fn shows(self, reference: &PictureReference, remote: &str) -> String {
        match self {
            Self::MarkdownPicture => format!("![{}]({remote})", escaped(&reference.description)),
            Self::AttachmentMacro => format!("!{remote}!"),
            Self::AttachmentLink => format!("[{}]({remote})", reference.picture.file_name()),
        }
    }
}

/// `uploads` maps a picture's path to how the manager refers to its copy.
pub fn pictures_as_uploads(markdown: &str, markup: PictureMarkup, uploads: &HashMap<String, String>) -> String {
    PictureReference::replaced_in(markdown, |reference| match uploads.get(reference.picture.relative()) {
        Some(remote) => markup.shows(reference, remote),
        None => description_of(reference),
    })
}

pub fn pictures_as_descriptions(markdown: &str) -> String {
    PictureReference::replaced_in(markdown, description_of)
}

fn description_of(reference: &PictureReference) -> String {
    let description = reference.description.trim();
    if description.is_empty() { reference.picture.file_name() } else { description }.to_string()
}

/// A description cannot close the picture's brackets and point it elsewhere.
fn escaped(description: &str) -> String {
    description.replace('\\', "\\\\").replace('[', "\\[").replace(']', "\\]")
}

#[cfg(test)]
#[path = "../../../tests/unit/connection/rule/tracker_markdown_tests.rs"]
mod tests;
