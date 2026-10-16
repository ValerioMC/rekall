//! Tasks: listed, created, edited, reviewed and deleted from the console. Every change to a
//! description keeps the text it replaces as a revision, and moving a task to `DONE` stops its
//! timer.

use rekall_common::{jstr, Id, RekallError, Result};
use rekall_model::{project, task, RevisionKind, Slug, TaskStatus, TaskStepState};
use rekall_repository::repository as repo;
use rekall_service::review::TaskReviewService;
use rekall_service::revision::{RevisionTrigger, TaskRevisionService};
use rekall_service::timeentry::TimeEntryService;
use rekall_service::{in_read, in_write, Ctx, Tx};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use crate::catalog::Snapshot;
use crate::catalog::{TaskRequest, TaskResponse};

#[derive(Clone)]
pub struct TaskService {
    ctx: Ctx,
    review: TaskReviewService,
    revisions: TaskRevisionService,
    time_entries: TimeEntryService,
}

impl TaskService {
    pub fn new(ctx: Ctx, review: TaskReviewService, revisions: TaskRevisionService, time_entries: TimeEntryService) -> Self {
        Self { ctx, review, revisions, time_entries }
    }

    pub async fn list(&self, project_id: Option<Id>) -> Result<Vec<TaskResponse>> {
        in_read!(&self.ctx, |tx| {
            let found = match project_id {
                None => repo::task::list_in_catalog_order(tx.db()).await?,
                Some(project_id) => repo::task::list_of_project(tx.db(), project_id).await?,
            };
            let snapshot = Snapshot::for_tasks(tx.db(), &found).await?;
            found.iter().map(|task| snapshot.task_response(task)).collect::<Result<Vec<_>>>()
        })
    }

    pub async fn get(&self, id: Id) -> Result<TaskResponse> {
        in_read!(&self.ctx, |tx| {
            let task = repo::task::require(tx.db(), id).await?;
            response(&tx, &task).await
        })
    }

    pub async fn create(&self, request: TaskRequest) -> Result<TaskResponse> {
        in_write!(&self.ctx, |tx| {
            let label = Slug::of(request.label.as_deref())?;
            let project = require_owning_project(&tx, request.project_id).await?;
            let mut task = task::Model::new(label, trimmed_title(&request), project.id, self.ctx.now());
            apply_request(&tx, &mut task, &request).await?;
            task.validate()?;
            require_label_free(&tx, &task, None).await?;
            task.clone().into_active_model().insert(tx.db()).await?;
            response(&tx, &task).await
        })
    }

    pub async fn update(&self, id: Id, request: TaskRequest) -> Result<TaskResponse> {
        in_write!(&self.ctx, |tx| {
            let before = repo::task::require(tx.db(), id).await?;
            self.keep_description_if_replaced(&mut tx, &before, request.description.as_deref(), RevisionTrigger::HandEdit)
                .await?;
            let mut task = before.clone();
            task.label = Slug::of(request.label.as_deref())?;
            task.title = trimmed_title(&request);
            task.project_id = require_owning_project(&tx, request.project_id).await?.id;
            apply_request(&tx, &mut task, &request).await?;
            require_label_free(&tx, &task, Some(id)).await?;
            let task = repo::update_if_changed(tx.db(), &before, task, self.ctx.now()).await?;
            let response = response(&tx, &task).await?;
            if before.status != TaskStatus::Done && task.status == TaskStatus::Done {
                self.time_entries.stop_if_running_in(&mut tx, id).await?;
            }
            Ok(response)
        })
    }

    /// Writes an earlier revision back as the task's description, keeping the one it replaces.
    pub async fn restore_description_in(&self, tx: &mut Tx, id: Id, description: &str) -> Result<TaskResponse> {
        let before = repo::task::require(tx.db(), id).await?;
        self.keep_description_if_replaced(tx, &before, Some(description), RevisionTrigger::Restore).await?;
        let mut task = before.clone();
        task.description = Some(description.to_string());
        let task = repo::update_if_changed(tx.db(), &before, task, self.ctx.now()).await?;
        response(tx, &task).await
    }

    /// `DONE` accepts a stepless task, `OPEN` sends it back; other states are refused.
    pub async fn review(&self, id: Id, target: TaskStepState, note: Option<&str>) -> Result<TaskResponse> {
        in_write!(&self.ctx, |tx| {
            repo::task::require(tx.db(), id).await?;
            match target {
                TaskStepState::Done => {
                    self.review.accept_in(&mut tx, id).await?;
                }
                TaskStepState::Open => {
                    self.review.send_back_in(&mut tx, id, note).await?;
                }
                _ => {
                    return Err(RekallError::illegal(
                        "The console can accept a task (DONE) or send it back (OPEN). RUNNING follows a live session \
                         and CLAIMED is set when a Claude-authored wrapup lands.",
                    ))
                }
            }
            let task = repo::task::require(tx.db(), id).await?;
            response(&tx, &task).await
        })
    }

    pub async fn delete(&self, id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let task = repo::task::require(tx.db(), id).await?;
            task::Entity::delete_by_id(task.id).exec(tx.db()).await?;
            Ok::<_, RekallError>(())
        })
    }

    async fn keep_description_if_replaced(
        &self,
        tx: &mut Tx,
        task: &task::Model,
        incoming: Option<&str>,
        trigger: RevisionTrigger,
    ) -> Result<()> {
        if task.description.as_deref() == incoming {
            return Ok(());
        }
        self.revisions
            .keep_in(tx, task.id, RevisionKind::Description, task.description.as_deref(), None, None, trigger)
            .await?;
        Ok(())
    }
}

/// The fields a request sets on create and update alike, past the label, title and project.
async fn apply_request(tx: &Tx, task: &mut task::Model, request: &TaskRequest) -> Result<()> {
    task.description = request.description.clone();
    task.status = request.status.unwrap_or(TaskStatus::Todo);
    task.tag_id = match request.tag_id {
        None => None,
        Some(tag_id) => Some(
            repo::tag::find_by_id(tx.db(), tag_id)
                .await?
                .ok_or_else(|| RekallError::not_found("Tag", tag_id))?
                .id,
        ),
    };
    Ok(())
}

async fn response(tx: &Tx, task: &task::Model) -> Result<TaskResponse> {
    Snapshot::for_task(tx.db(), task).await?.task_response(task)
}

fn trimmed_title(request: &TaskRequest) -> String {
    jstr::trim(request.title.as_deref().unwrap_or_default()).to_string()
}

async fn require_owning_project(tx: &Tx, id: Option<Id>) -> Result<project::Model> {
    let Some(id) = id else {
        return Err(RekallError::not_found_msg("A task must belong to a project"));
    };
    repo::project::require(tx.db(), id).await
}

/// A label names one task within its project, or a `task:` anchor would be ambiguous.
async fn require_label_free(tx: &Tx, task: &task::Model, except: Option<Id>) -> Result<()> {
    if repo::task::label_taken(tx.db(), task.project_id, &task.label, except).await? {
        return Err(RekallError::conflict(
            "Another task on this project already uses that label. An anchor has to name one record.",
        ));
    }
    Ok(())
}
