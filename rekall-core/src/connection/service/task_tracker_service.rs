use std::sync::Arc;

use rekall_kernel::{Id, InternalFailure, RekallError, Result};
use rekall_store::repository as repo;
use rekall_store::{external_connection, project_connection, task, task_tracker_link, ConnectionKind, TaskKind};
use sea_orm::{ActiveModelTrait, IntoActiveModel, Set};
use tracing::{info, warn};

use crate::connection::{Secret, SecretStore, TaskTracker, TrackerEndpoint, TrackerFailure, TrackerIssue, TrackerLink};
use crate::{in_read, in_write, load, Ctx};

/// Writes a task to the project's task manager: its title when the task is created, and its
/// title and description only when the person asks. A failure is kept on the task.
#[derive(Clone)]
pub struct TaskTrackerService {
    ctx: Ctx,
    secrets: Arc<dyn SecretStore>,
    tracker: Arc<dyn TaskTracker>,
}

/// What a write found to send and where.
struct Delivery {
    task: task::Model,
    issue_key: Option<String>,
    binding: project_connection::Model,
    connection: external_connection::Model,
}

impl TaskTrackerService {
    pub fn new(ctx: Ctx, secrets: Arc<dyn SecretStore>, tracker: Arc<dyn TaskTracker>) -> Self {
        Self { ctx, secrets, tracker }
    }

    /// Creates the issue of a task that has none, with its title alone. A project without a task
    /// manager, an epic and a quick task are left alone.
    pub async fn create_missing(&self, task_id: Id) -> Result<()> {
        let Some(delivery) = self.read(task_id).await? else { return Ok(()) };
        if delivery.issue_key.is_some() {
            return Ok(());
        }
        let issue = TrackerIssue::titled(&delivery.task);
        self.write(&delivery, &issue).await
    }

    /// Writes the task's title and description to its issue, creating the issue when the task has
    /// none. The failure is kept on the task and also returned, so the person who asked sees it.
    pub async fn sync_content(&self, task_id: Id) -> Result<()> {
        let delivery = self.read(task_id).await?.ok_or_else(|| RekallError::illegal("This task has no task manager to write to."))?;
        let issue = TrackerIssue::of(&delivery.task);
        self.write(&delivery, &issue).await
    }

    /// Removes the issue a deleted task held from the project's task manager, which the task's own
    /// row can no longer name. A project that lost its task manager since has nothing to tell.
    pub async fn remove(&self, project_id: Id, issue_key: &str) -> Result<()> {
        let Some(endpoint) = self.endpoint_of_project(project_id).await? else { return Ok(()) };
        self.tracker.delete(&endpoint, issue_key).await.map_err(|failure| RekallError::conflict(failure_message(endpoint.kind, &failure)))?;
        info!(%project_id, issue = issue_key, "A deleted task's issue was removed from the project's task manager");
        Ok(())
    }

    /// The project's task manager with the credential read from the store; none when it has no task
    /// manager. The one place a credential is read for a pull request too.
    pub async fn endpoint_of_project(&self, project_id: Id) -> Result<Option<TrackerEndpoint>> {
        let manager = in_read!(&self.ctx, |tx| repo::project_connection::task_manager_of_project(tx.db(), project_id).await)?;
        match manager {
            Some((binding, connection)) => self.endpoint(&binding, &connection).await.map(Some),
            None => Ok(None),
        }
    }

    async fn read(&self, task_id: Id) -> Result<Option<Delivery>> {
        in_read!(&self.ctx, |tx| {
            let task = load::task_or_unknown(tx.db(), task_id).await?;
            if task.kind != TaskKind::Task || task.quick {
                return Ok(None);
            }
            let manager = repo::project_connection::task_manager_of_project(tx.db(), task.project_id).await?;
            let issue_key = repo::task_tracker_link::find_of_task(tx.db(), task.id).await?.and_then(|link| link.issue_key);
            Ok(manager.map(|(binding, connection)| Delivery { task, issue_key, binding, connection }))
        })
    }

    async fn write(&self, delivery: &Delivery, issue: &TrackerIssue) -> Result<()> {
        let task_id = delivery.task.id;
        match self.deliver(delivery, issue).await {
            Ok(created) => {
                info!(task_id = %task_id, "A task was written to the project's task manager");
                self.keep_written(task_id, created, issue.fingerprint()).await
            }
            Err(failure) => {
                warn!(%failure, task_id = %task_id, "A task could not be written to the project's task manager");
                self.keep_failure(task_id, failure.message().to_string()).await?;
                Err(failure)
            }
        }
    }

    /// The link of the issue it created, or none when it rewrote the issue the task already had.
    async fn deliver(&self, delivery: &Delivery, issue: &TrackerIssue) -> Result<Option<TrackerLink>> {
        let endpoint = self.endpoint(&delivery.binding, &delivery.connection).await?;
        let kind = delivery.connection.kind;
        match delivery.issue_key.as_deref() {
            Some(key) => self.tracker.update(&endpoint, key, issue).await.map(|()| None),
            None => self.tracker.create(&endpoint, issue).await.map(Some),
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

    async fn keep_written(&self, task_id: Id, created: Option<TrackerLink>, fingerprint: String) -> Result<()> {
        self.keep_link(task_id, |held| {
            let (issue_key, issue_url) = match created {
                Some(link) => (Some(link.key), Some(link.url)),
                None => (held.issue_key.clone(), held.issue_url.clone()),
            };
            task_tracker_link::Model { issue_key, issue_url, sync_error: None, synced_fingerprint: Some(fingerprint), ..held }
        })
        .await
    }

    async fn keep_failure(&self, task_id: Id, failure: String) -> Result<()> {
        self.keep_link(task_id, |held| task_tracker_link::Model { sync_error: Some(failure), ..held }).await
    }

    /// Saves what the push found on the task's link, creating it the first time, and moves the
    /// task's own stamp when the link changed, since the console orders what it receives by it.
    async fn keep_link(&self, task_id: Id, change: impl FnOnce(task_tracker_link::Model) -> task_tracker_link::Model) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let task = load::task_or_unknown(tx.db(), task_id).await?;
            let now = self.ctx.now();
            let existing = repo::task_tracker_link::find_of_task(tx.db(), task_id).await?;
            let held = existing.clone().unwrap_or(task_tracker_link::Model { task_id, issue_key: None, issue_url: None, sync_error: None, synced_fingerprint: None, created_at: now, updated_at: now });
            let changed = change(held.clone());
            if existing.as_ref() == Some(&changed) || (existing.is_none() && changed == held) {
                return Ok(());
            }
            repo::task_tracker_link::save(tx.db(), task_tracker_link::Model { updated_at: now, ..changed }).await?;
            let mut stamped = task.into_active_model();
            stamped.updated_at = Set(now);
            stamped.update(tx.db()).await?;
            Ok(())
        })
    }
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
