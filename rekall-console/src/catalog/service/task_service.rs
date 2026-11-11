//! Tasks: listed, created, edited, reviewed and deleted from the console. Moving a task to `DONE`
//! stops its timer. An epic is a task of kind `EPIC`: it holds no steps, and keeps its kind and
//! its project while tasks belong to it.

use rekall_kernel::{jstr, Id, RekallError, Result};
use rekall_store::{project, task, Slug, TaskKind, TaskStatus, TaskStepState};
use rekall_store::repository as repo;
use rekall_core::review::TaskReviewService;
use rekall_core::task::TaskContentChangedEvent;
use rekall_core::timeentry::TimeEntryService;
use rekall_core::{in_read, in_write, Ctx, Tx};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use crate::catalog::Snapshot;
use crate::catalog::{TaskCommand, TaskView};
use crate::feedback::FeedbackService;

#[derive(Clone)]
pub struct TaskService {
    ctx: Ctx,
    review: TaskReviewService,
    time_entries: TimeEntryService,
    feedback: FeedbackService,
}

impl TaskService {
    pub fn new(ctx: Ctx, review: TaskReviewService, time_entries: TimeEntryService) -> Self {
        let feedback = FeedbackService::new(ctx.clone());
        Self { ctx, review, time_entries, feedback }
    }

    pub async fn list(&self, project_id: Option<Id>) -> Result<Vec<TaskView>> {
        in_read!(&self.ctx, |tx| {
            let found = match project_id {
                None => repo::task::list_in_catalog_order(tx.db()).await?,
                Some(project_id) => repo::task::list_of_project(tx.db(), project_id).await?,
            };
            let snapshot = Snapshot::for_tasks(tx.db(), &found).await?;
            found.iter().map(|task| snapshot.task_response(task)).collect::<Result<Vec<_>>>()
        })
    }

    pub async fn get(&self, id: Id) -> Result<TaskView> {
        in_read!(&self.ctx, |tx| {
            let task = repo::task::require(tx.db(), id).await?;
            response(&tx, &task).await
        })
    }

    pub async fn create(&self, command: TaskCommand) -> Result<TaskView> {
        in_write!(&self.ctx, |tx| self.create_in(&mut tx, command).await)
    }

    /// A task that arrives with the issue it was imported from is not pushed back to its task manager.
    pub async fn create_in(&self, tx: &mut Tx, command: TaskCommand) -> Result<TaskView> {
        let label = Slug::of(command.label.as_deref())?;
        let project = require_owning_project(tx, command.project_id).await?;
        let mut task = task::Model::new(label, trimmed_title(&command), project.id, self.ctx.now());
        apply_request(tx, &mut task, &command).await?;
        task.quick = command.quick;
        task.tracker_issue_key = command.tracker_issue_key.clone();
        task.tracker_issue_url = command.tracker_issue_url.clone();
        task.validate()?;
        require_label_free(tx, &task, None).await?;
        require_no_other_active_quick(tx, &task).await?;
        task.clone().into_active_model().insert(tx.db()).await?;
        if task.tracker_issue_key.is_none() {
            tx.publish(TaskContentChangedEvent { task_id: task.id });
        }
        response(tx, &task).await
    }

    pub async fn update(&self, id: Id, command: TaskCommand) -> Result<TaskView> {
        in_write!(&self.ctx, |tx| {
            let before = repo::task::require(tx.db(), id).await?;
            let mut task = before.clone();
            task.label = Slug::of(command.label.as_deref())?;
            task.title = trimmed_title(&command);
            task.project_id = require_owning_project(&tx, command.project_id).await?.id;
            apply_request(&tx, &mut task, &command).await?;
            require_label_free(&tx, &task, Some(id)).await?;
            require_epic_kept_whole(&tx, &before, &task).await?;
            require_epic_stepless(&tx, &task).await?;
            let task = repo::update_if_changed(tx.db(), &before, task, self.ctx.now()).await?;
            if before.title != task.title || before.description != task.description {
                tx.publish(TaskContentChangedEvent { task_id: id });
            }
            if before.description != task.description {
                self.feedback.track_description_in(&mut tx, &task, before.description.as_deref(), task.description.as_deref()).await?;
            }
            if before.status != TaskStatus::Done && task.status == TaskStatus::Done {
                self.time_entries.stop_if_running_in(&mut tx, id).await?;
                self.review.accept_everything_in(&mut tx, id).await?;
            }
            let task = repo::task::require(tx.db(), id).await?;
            response(&tx, &task).await
        })
    }

    /// `DONE` accepts a stepless task, `OPEN` sends it back; other states are refused.
    pub async fn review(&self, id: Id, target: TaskStepState) -> Result<TaskView> {
        in_write!(&self.ctx, |tx| {
            repo::task::require(tx.db(), id).await?;
            match target {
                TaskStepState::Done => {
                    self.review.accept_in(&mut tx, id).await?;
                }
                TaskStepState::Open => {
                    self.review.send_back_in(&mut tx, id).await?;
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

    /// Leaves the task unreleased when the console runs it, or lets the project's release graph be laid again.
    pub async fn skip_release(&self, id: Id, skipped: bool) -> Result<TaskView> {
        in_write!(&self.ctx, |tx| {
            let before = repo::task::require(tx.db(), id).await?;
            let task = task::Model { release_skipped: skipped, ..before.clone() };
            let task = repo::update_if_changed(tx.db(), &before, task, self.ctx.now()).await?;
            response(&tx, &task).await
        })
    }

    /// Records that a deployment graph was laid on the task, inside a transaction the caller holds.
    pub async fn mark_graph_laid_in(&self, tx: &Tx, id: Id) -> Result<()> {
        let before = repo::task::require(tx.db(), id).await?;
        let task = task::Model { graph_laid_at: Some(self.ctx.now()), ..before.clone() };
        repo::update_if_changed(tx.db(), &before, task, self.ctx.now()).await?;
        Ok(())
    }

    pub async fn delete(&self, id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let task = repo::task::require(tx.db(), id).await?;
            task::Entity::delete_by_id(task.id).exec(tx.db()).await?;
            Ok::<_, RekallError>(())
        })
    }
}

/// The fields a request sets on create and update alike, past the label, title and project.
async fn apply_request(tx: &Tx, task: &mut task::Model, command: &TaskCommand) -> Result<()> {
    task.description = command.description.clone();
    task.status = command.status.unwrap_or(TaskStatus::Todo);
    task.kind = command.kind.unwrap_or(TaskKind::Task);
    task.epic_id = match command.epic_id {
        None => None,
        Some(epic_id) => Some(require_joinable_epic(tx, task, epic_id).await?.id),
    };
    task.tag_id = match command.tag_id {
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

async fn response(tx: &Tx, task: &task::Model) -> Result<TaskView> {
    Snapshot::for_task(tx.db(), task).await?.task_response(task)
}

fn trimmed_title(command: &TaskCommand) -> String {
    jstr::trim(command.title.as_deref().unwrap_or_default()).to_string()
}

async fn require_owning_project(tx: &Tx, id: Option<Id>) -> Result<project::Model> {
    let Some(id) = id else {
        return Err(RekallError::not_found_msg("A task must belong to a project"));
    };
    repo::project::require(tx.db(), id).await
}

async fn require_joinable_epic(tx: &Tx, task: &task::Model, epic_id: Id) -> Result<task::Model> {
    let epic = repo::task::require(tx.db(), epic_id).await?;
    task.require_joinable(&epic)?;
    Ok(epic)
}

/// An epic that tasks belong to stays an epic and stays in their project, or they would point at
/// a plain task or across projects.
async fn require_epic_kept_whole(tx: &Tx, before: &task::Model, task: &task::Model) -> Result<()> {
    let regrouped = !task.is_epic() || task.project_id != before.project_id;
    if !before.is_epic() || !regrouped {
        return Ok(());
    }
    if repo::task::list_children_of(tx.db(), before.id).await?.is_empty() {
        return Ok(());
    }
    Err(RekallError::conflict(
        "Tasks belong to this epic. Take them out of it before turning it into a task or moving it to another project.",
    ))
}

/// An epic is a container for a description and its tasks, never a checklist of its own.
async fn require_epic_stepless(tx: &Tx, task: &task::Model) -> Result<()> {
    if !task.is_epic() || repo::task_step::list_of_task(tx.db(), task.id).await?.is_empty() {
        return Ok(());
    }
    Err(RekallError::conflict("This task has steps, and an epic holds none. Delete its steps before turning it into an epic."))
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

/// A quick task skips the review, so a project holds one at a time: it is for small changes, not a
/// way round the review for large ones.
async fn require_no_other_active_quick(tx: &Tx, task: &task::Model) -> Result<()> {
    if !task.quick || !repo::task::active_quick_exists(tx.db(), task.project_id).await? {
        return Ok(());
    }
    Err(RekallError::conflict(
        "This project already has a quick task being worked. Wait for its wrapup, or use a regular task for this change.",
    ))
}

#[cfg(test)]
#[path = "../../../tests/unit/catalog/service/task_service_tests.rs"]
mod tests;
