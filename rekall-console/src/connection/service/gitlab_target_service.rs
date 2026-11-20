//! The group or project a GitLab binding names, asked of the instance with the connection's own
//! credential: confirmed before a binding is kept, and listed so a person can choose it instead of
//! typing it. The instance is asked before any transaction opens.

use std::sync::Arc;

use rekall_core::connection::{GitlabNamespaces, SecretStore, TrackerEndpoint, TrackerFailure};
use rekall_core::{in_read, Ctx};
use rekall_kernel::{Id, RekallError, Result};
use rekall_store::repository as repo;
use rekall_store::{external_connection, ConnectionKind};
use tracing::warn;

use crate::connection::rule::gitlab_path_rules::{is_within_root, nearest_sibling, parent_of, path_of};
use crate::connection::support::credential_keeping::recall;
use crate::connection::view::gitlab_namespace_view::GitlabNamespaceView;
use crate::connection::view::target_check_view::TargetCheckView;

const CHILDREN_LIMIT: u16 = 50;
const SIBLINGS_LIMIT: u16 = 100;

#[derive(Clone)]
pub struct GitlabTargetService {
    ctx: Ctx,
    secrets: Arc<dyn SecretStore>,
    gitlab: Arc<dyn GitlabNamespaces>,
}

impl GitlabTargetService {
    pub fn new(ctx: Ctx, secrets: Arc<dyn SecretStore>, gitlab: Arc<dyn GitlabNamespaces>) -> Self {
        Self { ctx, secrets, gitlab }
    }

    /// What GitLab has at `raw_path`, a pasted address or a path, on the connection with `connection_id`.
    pub async fn check(&self, connection_id: Id, raw_path: &str) -> Result<TargetCheckView> {
        let connection = self.gitlab_connection(connection_id).await?;
        self.check_on(&connection, raw_path).await
    }

    /// The subgroups and projects directly under `parent` (the connection's root path when none is given) whose path holds `query`.
    pub async fn children(&self, connection_id: Id, parent: Option<&str>, query: Option<&str>) -> Result<Vec<GitlabNamespaceView>> {
        let connection = self.gitlab_connection(connection_id).await?;
        let root = root_of(&connection);
        let parent = parent.map(path_of).filter(|parent| !parent.is_empty()).or(root.clone()).unwrap_or_default();
        if !is_within_root(root.as_deref(), &parent) {
            return Err(RekallError::illegal(format!("'{parent}' is outside the root path of this connection.")));
        }
        let endpoint = self.endpoint_of(&connection, &parent).await?;
        let children = self.gitlab.children(&endpoint, query.unwrap_or_default().trim(), CHILDREN_LIMIT).await.map_err(refused_by_gitlab)?;
        Ok(children.into_iter().map(Into::into).collect())
    }

    /// Refuses a target GitLab does not have. A connection with no credential cannot be asked, so its target passes unchecked.
    pub async fn require_existing(&self, connection: &external_connection::Model, target: &str) -> Result<()> {
        if !connection.has_credential() {
            return Ok(());
        }
        match self.check_on(connection, target).await? {
            TargetCheckView::Found(_) | TargetCheckView::NoCredential => Ok(()),
            TargetCheckView::NotFound { nearest } => Err(RekallError::illegal(match nearest {
                Some(path) => format!("GitLab has no project or group '{}'. Did you mean '{path}'?", path_of(target)),
                None => format!("GitLab has no project or group '{}'. Pick one the connection offers.", path_of(target)),
            })),
            TargetCheckView::OutsideRoot { root } => Err(RekallError::illegal(format!("'{}' is outside the root path '{root}' of this connection.", path_of(target)))),
            TargetCheckView::Unreachable { reason } => Err(RekallError::conflict(reason)),
        }
    }

    async fn check_on(&self, connection: &external_connection::Model, raw_path: &str) -> Result<TargetCheckView> {
        let path = path_of(raw_path);
        if path.is_empty() {
            return Err(RekallError::illegal("Type the path of a GitLab group or project to check."));
        }
        let root = root_of(connection);
        if !is_within_root(root.as_deref(), &path) {
            return Ok(TargetCheckView::OutsideRoot { root: root.unwrap_or_default() });
        }
        if !connection.has_credential() {
            return Ok(TargetCheckView::NoCredential);
        }
        let endpoint = self.endpoint_of(connection, &path).await?;
        Ok(match self.gitlab.lookup(&endpoint).await {
            Ok(Some(found)) => TargetCheckView::Found(found.into()),
            Ok(None) => TargetCheckView::NotFound { nearest: self.nearest_to(connection, &path).await },
            Err(failure) => TargetCheckView::Unreachable { reason: reason_of(&failure) },
        })
    }

    async fn nearest_to(&self, connection: &external_connection::Model, path: &str) -> Option<String> {
        let parent = parent_of(path)?;
        if !is_within_root(root_of(connection).as_deref(), parent) {
            return None;
        }
        let endpoint = self.endpoint_of(connection, parent).await.ok()?;
        match self.gitlab.children(&endpoint, "", SIBLINGS_LIMIT).await {
            Ok(siblings) => nearest_sibling(path, &siblings),
            Err(failure) => {
                warn!(?failure, parent, "GitLab did not list the siblings of a path that was not found; no nearest path is offered");
                None
            }
        }
    }

    async fn gitlab_connection(&self, connection_id: Id) -> Result<external_connection::Model> {
        let connection = in_read!(&self.ctx, |tx| repo::external_connection::require(tx.db(), connection_id).await)?;
        if connection.kind != ConnectionKind::Gitlab {
            return Err(RekallError::illegal("Only a GitLab connection has groups and projects to check."));
        }
        Ok(connection)
    }

    async fn endpoint_of(&self, connection: &external_connection::Model, location: &str) -> Result<TrackerEndpoint> {
        if !connection.has_credential() {
            return Err(RekallError::illegal("No credential is stored for this GitLab connection. Set one on the Connections page."));
        }
        let credential = recall(self.secrets.clone(), connection.secret_ref.clone()).await?;
        Ok(TrackerEndpoint {
            kind: connection.kind,
            site_url: connection.base_url.clone().unwrap_or_default(),
            proxy_url: connection.proxy_url.clone(),
            location: location.to_string(),
            credential,
        })
    }
}

fn root_of(connection: &external_connection::Model) -> Option<String> {
    connection.root_path.as_deref().map(|root| root.trim_matches('/').to_string()).filter(|root| !root.is_empty())
}

fn reason_of(failure: &TrackerFailure) -> String {
    match failure {
        TrackerFailure::Rejected { status } => format!("GitLab rejected the stored credential (HTTP {status}). Replace it on the Connections page."),
        TrackerFailure::Failed { reason } => format!("GitLab could not be asked: {reason}."),
    }
}

fn refused_by_gitlab(failure: TrackerFailure) -> RekallError {
    RekallError::conflict(reason_of(&failure))
}

#[cfg(test)]
#[path = "../../../tests/unit/connection/service/gitlab_target_service_tests.rs"]
mod tests;
