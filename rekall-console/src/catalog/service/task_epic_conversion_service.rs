//! Turns a task into an epic. The title, code, status, description, notes, label, issue and time
//! stay; what an epic cannot hold is deleted in the same transaction. Nothing is converted while
//! the task waits on the run queue.

use rekall_kernel::{Id, RekallError, Result};
use rekall_store::repository as repo;
use rekall_store::{task, TaskKind, TaskStepState};
use rekall_core::timeentry::TimeEntryService;
use rekall_core::{in_read, in_write, Ctx, Tx};

use crate::catalog::{EpicConversionView, Snapshot, TaskView};

#[derive(Clone)]
pub struct TaskEpicConversionService {
    ctx: Ctx,
    time_entries: TimeEntryService,
}

impl TaskEpicConversionService {
    pub fn new(ctx: Ctx, time_entries: TimeEntryService) -> Self {
        Self { ctx, time_entries }
    }

    pub async fn preview(&self, id: Id) -> Result<EpicConversionView> {
        in_read!(&self.ctx, |tx| {
            let task = repo::task::require(tx.db(), id).await?;
            lost_by(&tx, &task).await
        })
    }

    pub async fn convert(&self, id: Id) -> Result<TaskView> {
        in_write!(&self.ctx, |tx| {
            let before = repo::task::require(tx.db(), id).await?;
            require_convertible(&tx, &before).await?;
            self.time_entries.stop_if_running_in(&mut tx, id).await?;
            delete_what_an_epic_cannot_hold(&tx, id).await?;
            let epic = repo::update_if_changed(tx.db(), &before, as_epic(&before), self.ctx.now()).await?;
            Snapshot::for_task(tx.db(), &epic).await?.task_response(&epic)
        })
    }
}

async fn lost_by(tx: &Tx, task: &task::Model) -> Result<EpicConversionView> {
    let steps = repo::task_step::list_of_task(tx.db(), task.id).await?;
    let dependencies = repo::task_dependency::list_involving(tx.db(), task.id).await?;
    let waits_for = dependencies.iter().filter(|dependency| dependency.task_id == task.id).count();
    let epic = match task.epic_id {
        Some(epic_id) => Some(repo::task::require(tx.db(), epic_id).await?),
        None => None,
    };
    Ok(EpicConversionView {
        task_id: task.id,
        steps: steps.len(),
        steps_done: steps.iter().filter(|step| step.state == TaskStepState::Done).count(),
        routes: repo::step_link::list_of_task(tx.db(), task.id).await?.len(),
        has_wrapup: repo::wrapup::find_of_task(tx.db(), task.id).await?.is_some(),
        commits: repo::commit_reference::list_of_task(tx.db(), task.id).await?.len(),
        deviations: repo::task_deviation::list_of_task(tx.db(), task.id).await?.len(),
        waits_for,
        needed_by: dependencies.len() - waits_for,
        skills: repo::task_skill::list_of_task(tx.db(), task.id).await?.len(),
        has_pull_request: repo::task_pull_request::find_of_task(tx.db(), task.id).await?.is_some(),
        epic_code: epic.as_ref().map(|epic| epic.code.clone()),
        epic_title: epic.map(|epic| epic.title),
        queued: repo::run_queue::task_is_queued(tx.db(), task.id).await?,
    })
}

async fn require_convertible(tx: &Tx, task: &task::Model) -> Result<()> {
    if task.is_epic() {
        return Err(RekallError::conflict(format!("'{}' is already an epic.", task.code)));
    }
    if repo::run_queue::task_is_queued(tx.db(), task.id).await? {
        return Err(RekallError::conflict(format!("'{}' is on the run queue. Take it off before turning it into an epic.", task.code)));
    }
    Ok(())
}

async fn delete_what_an_epic_cannot_hold(tx: &Tx, id: Id) -> Result<()> {
    repo::task_step::delete_of_task(tx.db(), id).await?;
    repo::wrapup::delete_of_task(tx.db(), id).await?;
    repo::commit_reference::delete_of_task(tx.db(), id).await?;
    repo::task_deviation::delete_of_task(tx.db(), id).await?;
    repo::task_dependency::delete_involving(tx.db(), id).await?;
    repo::task_skill::delete_of_task(tx.db(), id).await?;
    repo::task_pull_request::delete_of_task(tx.db(), id).await?;
    Ok(())
}

fn as_epic(task: &task::Model) -> task::Model {
    task::Model {
        kind: TaskKind::Epic,
        epic_id: None,
        review_state: TaskStepState::Open,
        claimed_at: None,
        accepted_at: None,
        auto_accepted: false,
        graph_laid_at: None,
        release_skipped: false,
        quick: false,
        worktree_folder: None,
        last_work_provider: None,
        ..task.clone()
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/catalog/service/task_epic_conversion_service_tests.rs"]
mod tests;
