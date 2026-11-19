//! A Jira connection's issues as one project's binding sees them: the open ones listed, and the
//! chosen ones imported as the project's tasks. Jira is asked before any transaction opens.

use std::collections::HashMap;
use std::sync::Arc;

use rekall_kernel::{Id, RekallError, Result};
use rekall_store::repository as repo;
use rekall_store::ConnectionKind;
use rekall_core::connection::{JiraIssue, JiraIssues, SecretStore};
use rekall_core::{in_read, in_write, Ctx};

use crate::connection::support::atlassian_access::{access_of, refused_by, AtlassianAccess};
use crate::connection::rule::jira_rules::{chosen_issues_query, open_issues_query, project_key_of, task_label, task_request, ISSUE_LIMIT};
use crate::connection::{JiraImportCommand, JiraIssueView, ProjectConnectionService};
use crate::catalog::{TaskView, TaskService};

#[derive(Clone)]
pub struct JiraIssueService {
    ctx: Ctx,
    bindings: ProjectConnectionService,
    tasks: TaskService,
    secrets: Arc<dyn SecretStore>,
    jira: Arc<dyn JiraIssues>,
}

/// Where a binding's issues are read: the site, the project key and the credential that opens it.
struct JiraSource {
    access: AtlassianAccess,
    project_key: String,
    task_manager: bool,
}

impl JiraIssueService {
    pub fn new(ctx: Ctx, bindings: ProjectConnectionService, tasks: TaskService, secrets: Arc<dyn SecretStore>, jira: Arc<dyn JiraIssues>) -> Self {
        Self { ctx, bindings, tasks, secrets, jira }
    }

    pub async fn list(&self, project_id: Id, binding_id: Id) -> Result<Vec<JiraIssueView>> {
        let source = self.source(project_id, binding_id).await?;
        let issues = self.search(&source, &open_issues_query(&source.project_key), ISSUE_LIMIT).await?;
        let imported = self.imported_tasks(project_id).await?;
        Ok(issues.iter().map(|issue| JiraIssueView::of(issue, &source.access.site.url, imported.get(&task_label(&issue.key)).copied())).collect())
    }

    /// Issues already imported are passed over, so importing twice adds nothing.
    pub async fn import(&self, project_id: Id, binding_id: Id, command: JiraImportCommand) -> Result<Vec<TaskView>> {
        let source = self.source(project_id, binding_id).await?;
        let query = chosen_issues_query(&source.project_key, &command.keys)?;
        let issues = self.search(&source, &query, ISSUE_LIMIT).await?;
        let imported = self.imported_tasks(project_id).await?;
        let fresh: Vec<&JiraIssue> = issues.iter().filter(|issue| !imported.contains_key(&task_label(&issue.key))).collect();
        in_write!(&self.ctx, |tx| {
            let mut created = Vec::with_capacity(fresh.len());
            for issue in &fresh {
                created.push(self.tasks.create_in(&mut tx, task_request(project_id, &source.access.site.url, issue, source.task_manager)).await?);
            }
            Ok(created)
        })
    }

    async fn source(&self, project_id: Id, binding_id: Id) -> Result<JiraSource> {
        let (binding, connection) = self.bindings.binding(project_id, binding_id).await?;
        if connection.kind != ConnectionKind::Jira {
            return Err(RekallError::illegal("Only a Jira connection has issues to import."));
        }
        let project_key = project_key_of(binding.target.as_deref())?;
        let access = access_of(self.secrets.clone(), &connection, "Jira").await?;
        Ok(JiraSource { access, project_key, task_manager: binding.task_manager })
    }

    async fn search(&self, source: &JiraSource, jql: &str, limit: u16) -> Result<Vec<JiraIssue>> {
        self.jira.search(&source.access.site, &source.access.credential, jql, limit).await.map_err(|failure| refused_by("Jira", failure))
    }

    /// The project's task ids by label.
    async fn imported_tasks(&self, project_id: Id) -> Result<HashMap<String, Id>> {
        in_read!(&self.ctx, |tx| {
            let tasks = repo::task::list_of_project(tx.db(), project_id).await?;
            Ok::<_, RekallError>(tasks.into_iter().map(|task| (task.label, task.id)).collect())
        })
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/connection/service/jira_issue_service_tests.rs"]
mod tests;
