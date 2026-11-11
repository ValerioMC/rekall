use std::sync::Arc;

use rekall_kernel::{Id, InternalFailure, RekallError, Result};
use rekall_store::repository as repo;
use rekall_store::{external_connection, project_connection, task, ConnectionKind, TaskKind};
use tracing::{info, warn};

use crate::connection::{Secret, SecretStore, TaskTracker, TrackerEndpoint, TrackerFailure, TrackerIssue, TrackerLink};
use crate::{in_read, in_write, load, Ctx};

const ISSUE_TITLE_MAX_CHARACTERS: usize = 200;

/// Writes a task to the project's task manager: the issue is created when the task has none and
/// rewritten when it has. A failure is kept on the task and retried by its next change.
#[derive(Clone)]
pub struct TaskTrackerService {
    ctx: Ctx,
    secrets: Arc<dyn SecretStore>,
    tracker: Arc<dyn TaskTracker>,
}

/// What a push found to write and where.
struct Delivery {
    task: task::Model,
    binding: project_connection::Model,
    connection: external_connection::Model,
}

impl TaskTrackerService {
    pub fn new(ctx: Ctx, secrets: Arc<dyn SecretStore>, tracker: Arc<dyn TaskTracker>) -> Self {
        Self { ctx, secrets, tracker }
    }

    /// A project without a task manager, an epic and a quick task are left alone.
    pub async fn push(&self, task_id: Id) -> Result<()> {
        let Some(delivery) = self.read(task_id).await? else { return Ok(()) };
        match self.deliver(&delivery).await {
            Ok(Some(link)) => {
                info!(task_id = %task_id, issue = %link.key, "A task was created in the project's task manager");
                self.keep_issue(task_id, link).await
            }
            Ok(None) => self.keep_failure(task_id, None).await,
            Err(failure) => {
                warn!(%failure, task_id = %task_id, "A task could not be written to the project's task manager");
                self.keep_failure(task_id, Some(failure.message().to_string())).await
            }
        }
    }

    async fn read(&self, task_id: Id) -> Result<Option<Delivery>> {
        in_read!(&self.ctx, |tx| {
            let task = load::task_or_unknown(tx.db(), task_id).await?;
            if task.kind != TaskKind::Task || task.quick {
                return Ok(None);
            }
            let manager = repo::project_connection::task_manager_of_project(tx.db(), task.project_id).await?;
            Ok(manager.map(|(binding, connection)| Delivery { task, binding, connection }))
        })
    }

    /// The link of the issue it created, or none when it rewrote the issue the task already had.
    async fn deliver(&self, delivery: &Delivery) -> Result<Option<TrackerLink>> {
        let endpoint = self.endpoint(&delivery.binding, &delivery.connection).await?;
        let issue = issue_of(&delivery.task);
        let kind = delivery.connection.kind;
        match delivery.task.tracker_issue_key.as_deref() {
            Some(key) => self.tracker.update(&endpoint, key, &issue).await.map(|()| None),
            None => self.tracker.create(&endpoint, &issue).await.map(Some),
        }
        .map_err(|failure| RekallError::conflict(failure_message(kind, &failure)))
    }

    async fn endpoint(&self, binding: &project_connection::Model, connection: &external_connection::Model) -> Result<TrackerEndpoint> {
        if !connection.has_credential() {
            return Err(RekallError::illegal("No credential is stored for the task manager's connection. Set one on the Connections page."));
        }
        let secrets = self.secrets.clone();
        let account = connection.secret_ref.clone();
        let credential: Secret = tokio::task::spawn_blocking(move || secrets.read(&account))
            .await
            .map_err(|failure| RekallError::internal(InternalFailure::Background, failure))??;
        Ok(TrackerEndpoint {
            kind: connection.kind,
            site_url: connection.base_url.clone().unwrap_or_default(),
            proxy_url: connection.proxy_url.clone(),
            location: location_of(connection.kind, binding.target.as_deref())?,
            credential,
        })
    }

    async fn keep_issue(&self, task_id: Id, link: TrackerLink) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let before = load::task_or_unknown(tx.db(), task_id).await?;
            let after = task::Model { tracker_issue_key: Some(link.key), tracker_issue_url: Some(link.url), tracker_sync_error: None, ..before.clone() };
            repo::update_if_changed(tx.db(), &before, after, self.ctx.now()).await?;
            Ok(())
        })
    }

    async fn keep_failure(&self, task_id: Id, failure: Option<String>) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let before = load::task_or_unknown(tx.db(), task_id).await?;
            let after = task::Model { tracker_sync_error: failure, ..before.clone() };
            repo::update_if_changed(tx.db(), &before, after, self.ctx.now()).await?;
            Ok(())
        })
    }
}

fn issue_of(task: &task::Model) -> TrackerIssue {
    TrackerIssue { title: task.title.chars().take(ISSUE_TITLE_MAX_CHARACTERS).collect(), description: task.description.clone().unwrap_or_default() }
}

/// A Jira project key is written upper-case, a repository path as typed without its slashes at the ends.
fn location_of(kind: ConnectionKind, target: Option<&str>) -> Result<String> {
    let location = target.map(str::trim).map(|target| target.trim_matches('/')).unwrap_or_default();
    if location.is_empty() {
        return Err(RekallError::illegal("Set the task manager binding's target on the project's page."));
    }
    Ok(if kind == ConnectionKind::Jira { location.to_uppercase() } else { location.to_string() })
}

/// The tracker's refusal is the stored credential or the site, which the person fixes on the Connections page.
fn failure_message(kind: ConnectionKind, failure: &TrackerFailure) -> String {
    let name = match kind {
        ConnectionKind::Jira => "Jira",
        ConnectionKind::Gitlab => "GitLab",
        ConnectionKind::Github => "GitHub",
        ConnectionKind::Kubernetes | ConnectionKind::Grafana => "The task manager",
    };
    match failure {
        TrackerFailure::Rejected { status } => format!("{name} rejected the stored credential (HTTP {status}). Replace it on the Connections page."),
        TrackerFailure::Failed { reason } => format!("{name} could not take the issue: {reason}."),
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/connection/service/task_tracker_service_tests.rs"]
mod tests;
