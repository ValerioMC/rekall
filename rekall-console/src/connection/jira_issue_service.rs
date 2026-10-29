//! A Jira connection's issues as one project's binding sees them: the open ones listed, and the
//! chosen ones imported as the project's tasks. Jira is asked before any transaction opens.

use std::collections::HashMap;
use std::sync::Arc;

use rekall_common::{Id, RekallError, Result};
use rekall_store::ConnectionKind;
use rekall_service::connection::{JiraCredential, JiraFailure, JiraIssue, JiraIssues, JiraSite, SecretStore};
use rekall_service::{in_write, Ctx};

use super::credential_keeping::recall;
use super::jira_rules::{chosen_issues_query, open_issues_query, project_key_of, task_label, task_request, ISSUE_LIMIT};
use super::{JiraImportCommand, JiraIssueView, ProjectConnectionService};
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
    site: JiraSite,
    project_key: String,
    credential: JiraCredential,
}

impl JiraIssueService {
    pub fn new(ctx: Ctx, bindings: ProjectConnectionService, tasks: TaskService, secrets: Arc<dyn SecretStore>, jira: Arc<dyn JiraIssues>) -> Self {
        Self { ctx, bindings, tasks, secrets, jira }
    }

    pub async fn list(&self, project_id: Id, binding_id: Id) -> Result<Vec<JiraIssueView>> {
        let source = self.source(project_id, binding_id).await?;
        let issues = self.search(&source, &open_issues_query(&source.project_key), ISSUE_LIMIT).await?;
        let imported = self.imported_tasks(project_id).await?;
        Ok(issues.iter().map(|issue| JiraIssueView::of(issue, &source.site.url, imported.get(&task_label(&issue.key)).copied())).collect())
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
                created.push(self.tasks.create_in(&tx, task_request(project_id, &source.site.url, issue)).await?);
            }
            Ok(created)
        })
    }

    async fn source(&self, project_id: Id, binding_id: Id) -> Result<JiraSource> {
        let (binding, connection) = self.bindings.binding(project_id, binding_id).await?;
        if connection.kind != ConnectionKind::Jira {
            return Err(RekallError::illegal("Only a Jira connection has issues to import."));
        }
        if !connection.has_credential() {
            return Err(RekallError::illegal("No credential is stored for this Jira connection. Set one on the company's Connections page."));
        }
        let project_key = project_key_of(binding.target.as_deref())?;
        let stored = recall(self.secrets.clone(), connection.secret_ref.clone()).await?;
        Ok(JiraSource { site: JiraSite::of(&connection), project_key, credential: JiraCredential::from_stored(&stored)? })
    }

    async fn search(&self, source: &JiraSource, jql: &str, limit: u16) -> Result<Vec<JiraIssue>> {
        self.jira.search(&source.site, &source.credential, jql, limit).await.map_err(refused_by_jira)
    }

    /// The project's task ids by label.
    async fn imported_tasks(&self, project_id: Id) -> Result<HashMap<String, Id>> {
        Ok(self.tasks.list(Some(project_id)).await?.into_iter().map(|task| (task.label, task.id)).collect())
    }
}

/// Jira's refusal is the stored credential or the site, which the person fixes on the Connections page.
fn refused_by_jira(failure: JiraFailure) -> RekallError {
    match failure {
        JiraFailure::Rejected { status } => {
            RekallError::conflict(format!("Jira rejected the stored credential (HTTP {status}). Replace it on the company's Connections page."))
        }
        JiraFailure::Failed { reason } => RekallError::conflict(format!("Jira could not be asked: {reason}.")),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/connection/jira_issue_service_tests.rs"]
mod tests;
