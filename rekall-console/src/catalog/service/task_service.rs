//! Tasks: listed, created, edited, reviewed and deleted from the console. Moving a task to `DONE`
//! stops its timer. An epic is a task of kind `EPIC`: it holds no steps, and keeps its kind and
//! its project while tasks belong to it.

use rekall_kernel::{jstr, Id, RekallError, Result};
use rekall_store::{project, task, task_tracker_link, Slug, TaskKind, TaskStatus, TaskStepState};
use rekall_store::repository as repo;
use rekall_core::review::TaskReviewService;
use rekall_core::task::{TaskCreatedEvent, TaskDeletedEvent};
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
        let project = require_owning_project(tx, command.project_id).await?;
        let mut task = task::Model::new(String::new(), trimmed_title(&command), project.id, self.ctx.now());
        repo::task::assign_next_code(tx.db(), &mut task, &project).await?;
        if let Some(label) = command.label.as_deref() {
            task.label = Slug::of(Some(label))?;
            require_label_free(tx, &task).await?;
        }
        apply_request(tx, &mut task, &command).await?;
        task.quick = command.quick;
        task.validate()?;
        task.clone().into_active_model().insert(tx.db()).await?;
        if command.tracker_issue_key.is_none() {
            tx.publish(TaskCreatedEvent { task_id: task.id });
        } else {
            self.keep_imported_issue_in(tx, &task, &command).await?;
        }
        response(tx, &task).await
    }

    async fn keep_imported_issue_in(&self, tx: &mut Tx, task: &task::Model, command: &TaskCommand) -> Result<()> {
        let now = self.ctx.now();
        let link = task_tracker_link::Model {
            task_id: task.id,
            issue_key: command.tracker_issue_key.clone(),
            issue_url: command.tracker_issue_url.clone(),
            sync_error: None,
            synced_fingerprint: None,
            created_at: now,
            updated_at: now,
        };
        repo::task_tracker_link::save(tx.db(), link).await
    }

    pub async fn update(&self, id: Id, command: TaskCommand) -> Result<TaskView> {
        in_write!(&self.ctx, |tx| {
            let before = repo::task::require(tx.db(), id).await?;
            let mut task = before.clone();
            task.title = trimmed_title(&command);
            let project = require_owning_project(&tx, command.project_id).await?;
            task.project_id = project.id;
            apply_request(&tx, &mut task, &command).await?;
            number_again_when_moved(&tx, &before, &mut task, &project).await?;
            require_epic_kept_whole(&tx, &before, &task).await?;
            require_epic_stepless(&tx, &task).await?;
            let task = repo::update_if_changed(tx.db(), &before, task, self.ctx.now()).await?;
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

    /// `DONE` accepts a stepless task, or every claimed step of one with a checklist; `OPEN` sends a stepless task back; other states are refused.
    pub async fn review(&self, id: Id, target: TaskStepState) -> Result<TaskView> {
        in_write!(&self.ctx, |tx| {
            repo::task::require(tx.db(), id).await?;
            match target {
                TaskStepState::Done => {
                    self.review.accept_work_in(&mut tx, id).await?;
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
            let issue_key = repo::task_tracker_link::find_of_task(tx.db(), id).await?.and_then(|link| link.issue_key);
            task::Entity::delete_by_id(task.id).exec(tx.db()).await?;
            if issue_key.is_some() {
                tx.publish(TaskDeletedEvent { project_id: task.project_id, issue_key });
            }
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

/// An explicit label names one task within its project.
async fn require_label_free(tx: &Tx, task: &task::Model) -> Result<()> {
    if repo::task::label_taken(tx.db(), task.project_id, &task.label, None).await? {
        return Err(RekallError::conflict("Another task on this project already uses that label."));
    }
    Ok(())
}

/// A task that changes project takes the next number there, and the code that goes with it.
async fn number_again_when_moved(tx: &Tx, before: &task::Model, task: &mut task::Model, project: &project::Model) -> Result<()> {
    if before.project_id == task.project_id {
        return Ok(());
    }
    let number = repo::task::next_number(tx.db(), project.id).await?;
    task.assign_code(&project.task_key, number);
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/unit/catalog/service/task_service_tests.rs"]
mod tests;
