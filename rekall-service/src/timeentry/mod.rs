//! Time tracking: one session of work on a task at a time per task, corrected by hand within
//! rules that keep it sane.

use rekall_common::{Id, Instant, RekallError, Result};
use rekall_model::{project, task, time_entry};
use rekall_repository::repository as repo;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};
use serde::Serialize;

use crate::{in_read, in_write, load, Ctx, Tx};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeEntryView {
    pub id: Id,
    pub task_id: Id,
    pub task_label: String,
    pub task_title: String,
    pub project_label: String,
    pub anchor: String,
    pub started_at: Instant,
    pub stopped_at: Option<Instant>,
    pub created_at: Instant,
    pub updated_at: Instant,
}

impl TimeEntryView {
    pub fn of(entry: &time_entry::Model, task: &task::Model, project: &project::Model) -> Self {
        Self {
            id: entry.id,
            task_id: task.id,
            task_label: task.label.clone(),
            task_title: task.title.clone(),
            project_label: project.label.clone(),
            anchor: format!("project:{} task:{}", project.label, task.label),
            started_at: entry.started_at,
            stopped_at: entry.stopped_at,
            created_at: entry.created_at,
            updated_at: entry.updated_at,
        }
    }

    async fn load(db: &impl sea_orm::ConnectionTrait, entry: &time_entry::Model) -> Result<Self> {
        let task = load::task_or_unknown(db, entry.task_id).await?;
        let project = load::project_of(db, &task).await?;
        Ok(Self::of(entry, &task, &project))
    }
}

#[derive(Clone)]
pub struct TimeEntryService {
    ctx: Ctx,
}

impl TimeEntryService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn find_all(&self) -> Result<Vec<TimeEntryView>> {
        in_read!(&self.ctx, |tx| {
            let all = repo::time_entry::find_all_by_order_by_started_at_desc(tx.db()).await?;
            let tasks = repo::task::find_all(tx.db()).await?;
            let projects = repo::project::find_all(tx.db()).await?;
            let mut views = Vec::with_capacity(all.len());
            for entry in &all {
                let task = tasks.iter().find(|t| t.id == entry.task_id);
                let project = task.and_then(|t| projects.iter().find(|p| p.id == t.project_id));
                if let (Some(task), Some(project)) = (task, project) {
                    views.push(TimeEntryView::of(entry, task, project));
                }
            }
            Ok::<_, RekallError>(views)
        })
    }

    pub async fn start(&self, task_id: Id) -> Result<TimeEntryView> {
        in_write!(&self.ctx, |tx| {
            load::task_or_unknown(tx.db(), task_id).await?;
            if let Some(running) = repo::time_entry::find_by_task_id_and_stopped_at_is_null(tx.db(), task_id).await? {
                return TimeEntryView::load(tx.db(), &running).await;
            }
            let now = self.ctx.now();
            let created = time_entry::Model {
                id: Id::random(),
                task_id,
                started_at: now,
                stopped_at: None,
                created_at: now,
                updated_at: now,
            };
            created.clone().into_active_model().insert(tx.db()).await?;
            TimeEntryView::load(tx.db(), &created).await
        })
    }

    pub async fn stop(&self, task_id: Id) -> Result<TimeEntryView> {
        in_write!(&self.ctx, |tx| {
            load::task_or_unknown(tx.db(), task_id).await?;
            self.stop_if_running_in(&mut tx, task_id)
                .await?
                .ok_or_else(|| RekallError::illegal("Nothing is being tracked on this task."))
        })
    }

    pub async fn stop_if_running_in(&self, tx: &mut Tx, task_id: Id) -> Result<Option<TimeEntryView>> {
        let Some(running) = repo::time_entry::find_by_task_id_and_stopped_at_is_null(tx.db(), task_id).await? else {
            return Ok(None);
        };
        let now = self.ctx.now();
        let mut stopped = running.clone();
        stopped.stopped_at = Some(now);
        stopped.updated_at = now;
        stopped.clone().into_active_model().reset_all().update(tx.db()).await?;
        Ok(Some(TimeEntryView::load(tx.db(), &stopped).await?))
    }

    pub async fn edit(&self, id: Id, started_at: Option<Instant>, stopped_at: Option<Instant>) -> Result<TimeEntryView> {
        in_write!(&self.ctx, |tx| {
            let entry = repo::time_entry::find_by_id(tx.db(), id)
                .await?
                .ok_or_else(|| RekallError::unknown_anchor(format!("No time entry with id {id}")))?;
            let Some(started_at) = started_at else {
                return Err(RekallError::illegal("A session needs a start time."));
            };
            if stopped_at.is_none() && entry.stopped_at.is_some() {
                return Err(RekallError::illegal(
                    "A finished session can't be reopened. Delete it and start a new one instead.",
                ));
            }
            if let Some(stopped_at) = stopped_at {
                if !stopped_at.is_after(&started_at) {
                    return Err(RekallError::illegal("A session has to end after it starts."));
                }
            }
            let mut edited = entry.clone();
            edited.started_at = started_at;
            edited.stopped_at = stopped_at;
            if edited != entry {
                edited.updated_at = self.ctx.now();
                edited.clone().into_active_model().reset_all().update(tx.db()).await?;
            }
            TimeEntryView::load(tx.db(), &edited).await
        })
    }

    pub async fn delete(&self, id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            time_entry::Entity::delete_by_id(id).exec(tx.db()).await?;
            Ok::<_, RekallError>(())
        })
    }

    /// Stop every open timer on shutdown so wall-clock duration, computed on read from
    /// `started_at`, doesn't silently keep growing across the app's downtime.
    pub async fn stop_all_on_shutdown(&self) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let now = self.ctx.now();
            for entry in repo::time_entry::find_all_by_stopped_at_is_null(tx.db()).await? {
                let mut stopped = entry;
                stopped.stopped_at = Some(now);
                stopped.updated_at = now;
                stopped.into_active_model().reset_all().update(tx.db()).await?;
            }
            Ok::<_, RekallError>(())
        })
    }
}
