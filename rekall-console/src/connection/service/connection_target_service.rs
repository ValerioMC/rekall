//! The target a binding names on Jira or Confluence, checked against the site before it is kept,
//! and the projects or spaces the site offers to fill it with. Any other connection's target is
//! free text and passes. The site is asked before any transaction opens.

use std::sync::Arc;

use rekall_core::connection::{ConfluenceSpaces, JiraIssues, SecretStore};
use rekall_kernel::{Id, RekallError, Result};
use rekall_store::{external_connection, ConnectionKind};
use rekall_store::repository as repo;
use rekall_core::{in_read, Ctx};

use crate::connection::rule::confluence_rules::{space_key_of, SUGGESTION_LIMIT};
use crate::connection::rule::connection_rules::target_of;
use crate::connection::rule::jira_rules::project_key_of;
use crate::connection::support::atlassian_access::{access_of, refused_by};
use crate::connection::view::target_suggestion_view::TargetSuggestionView;
use crate::connection::{ProjectConnectionCommand, ProjectConnectionService, ProjectConnectionView};

#[derive(Clone)]
pub struct ConnectionTargetService {
    ctx: Ctx,
    bindings: ProjectConnectionService,
    secrets: Arc<dyn SecretStore>,
    jira: Arc<dyn JiraIssues>,
    confluence: Arc<dyn ConfluenceSpaces>,
}

impl ConnectionTargetService {
    pub fn new(ctx: Ctx, bindings: ProjectConnectionService, secrets: Arc<dyn SecretStore>, jira: Arc<dyn JiraIssues>, confluence: Arc<dyn ConfluenceSpaces>) -> Self {
        Self { ctx, bindings, secrets, jira, confluence }
    }

    pub async fn bind(&self, project_id: Id, connection_id: Id, command: ProjectConnectionCommand) -> Result<ProjectConnectionView> {
        let connection = in_read!(&self.ctx, |tx| repo::external_connection::require(tx.db(), connection_id).await)?;
        self.require_existing(&connection, target_of(command.target.as_deref()).as_deref()).await?;
        self.bindings.bind(project_id, connection_id, command).await
    }

    pub async fn update(&self, project_id: Id, binding_id: Id, command: ProjectConnectionCommand) -> Result<ProjectConnectionView> {
        let (_, connection) = self.bindings.binding(project_id, binding_id).await?;
        self.require_existing(&connection, target_of(command.target.as_deref()).as_deref()).await?;
        self.bindings.update(project_id, binding_id, command).await
    }

    /// The projects or spaces whose key or name holds `query`, none for a connection that has no such list.
    pub async fn suggestions(&self, connection_id: Id, query: Option<&str>) -> Result<Vec<TargetSuggestionView>> {
        let connection = in_read!(&self.ctx, |tx| repo::external_connection::require(tx.db(), connection_id).await)?;
        let query = query.unwrap_or_default();
        match connection.kind {
            ConnectionKind::Jira => {
                let access = access_of(self.secrets.clone(), &connection, "Jira").await?;
                let projects = self.jira.projects(&access.site, &access.credential, query, SUGGESTION_LIMIT).await.map_err(|failure| refused_by("Jira", failure))?;
                Ok(projects.into_iter().map(|project| TargetSuggestionView { value: project.key, name: project.name }).collect())
            }
            ConnectionKind::Confluence => {
                let access = access_of(self.secrets.clone(), &connection, "Confluence").await?;
                let spaces = self.confluence.spaces(&access.site, &access.credential, query, SUGGESTION_LIMIT).await.map_err(|failure| refused_by("Confluence", failure))?;
                Ok(spaces.into_iter().map(|space| TargetSuggestionView { value: space.key, name: space.name }).collect())
            }
            ConnectionKind::Gitlab | ConnectionKind::Github | ConnectionKind::Kubernetes | ConnectionKind::Grafana => Ok(Vec::new()),
        }
    }

    async fn require_existing(&self, connection: &external_connection::Model, target: Option<&str>) -> Result<()> {
        let Some(target) = target else { return Ok(()) };
        match connection.kind {
            ConnectionKind::Jira => self.require_jira_project(connection, target).await,
            ConnectionKind::Confluence => self.require_confluence_space(connection, target).await,
            ConnectionKind::Gitlab | ConnectionKind::Github | ConnectionKind::Kubernetes | ConnectionKind::Grafana => Ok(()),
        }
    }

    async fn require_jira_project(&self, connection: &external_connection::Model, target: &str) -> Result<()> {
        let key = project_key_of(Some(target))?;
        let access = access_of(self.secrets.clone(), connection, "Jira").await?;
        let found = self.jira.project(&access.site, &access.credential, &key).await.map_err(|failure| refused_by("Jira", failure))?;
        found.map(|_| ()).ok_or_else(|| RekallError::illegal(format!("Jira has no project '{key}'. Pick one of the projects the site offers.")))
    }

    async fn require_confluence_space(&self, connection: &external_connection::Model, target: &str) -> Result<()> {
        let key = space_key_of(Some(target))?;
        let access = access_of(self.secrets.clone(), connection, "Confluence").await?;
        let found = self.confluence.space(&access.site, &access.credential, &key).await.map_err(|failure| refused_by("Confluence", failure))?;
        found.map(|_| ()).ok_or_else(|| RekallError::illegal(format!("Confluence has no space '{key}'. Pick one of the spaces the site offers.")))
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/connection/service/connection_target_service_tests.rs"]
mod tests;
