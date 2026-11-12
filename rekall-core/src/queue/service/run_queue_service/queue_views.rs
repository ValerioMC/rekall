//! What the console and the runner are shown of the list: each item with its task, its steps and
//! what it waits for.

use std::collections::HashSet;

use rekall_kernel::{Id, InternalFailure, RekallError, Result};
use rekall_store::repository as repo;
use rekall_store::{run_queue, run_queue_item, RunQueueItemState};

use crate::claude::TaskWorkService;
use crate::dependency::{TaskWaitService, AWAITING_REVIEW_TEXT};
use crate::queue::{RunQueueItemView, RunQueueView, RunQueueWaitView};
use crate::Tx;

use super::ordered;

pub(super) async fn item_view(tx: &Tx, item: &run_queue_item::Model, queued_tasks: &HashSet<Id>) -> Result<RunQueueItemView> {
    let task = repo::task::find_by_id(tx.db(), item.task_id)
        .await?
        .ok_or_else(|| RekallError::unknown_anchor(format!("No task with id {}", item.task_id)))?;
    let project = repo::project::find_by_id(tx.db(), task.project_id)
        .await?
        .ok_or_else(|| RekallError::internal(InternalFailure::Integrity, "A task has no project"))?;
    let steps = TaskWorkService::progress_in(tx, task.id).await?;
    let waiting_for = unmet_dependencies(tx, item, queued_tasks).await?;
    Ok(RunQueueItemView::of(item, &task, &project, steps, waiting_for))
}

pub(super) async fn unmet_dependencies(tx: &Tx, item: &run_queue_item::Model, queued_tasks: &HashSet<Id>) -> Result<Vec<RunQueueWaitView>> {
    if item.state != RunQueueItemState::Queued {
        return Ok(Vec::new());
    }
    Ok(TaskWaitService::of_task_in(tx, item.task_id)
        .await?
        .iter()
        .filter(|wait| !wait.is_met())
        .map(|wait| RunQueueWaitView::of(wait, queued_tasks.contains(&wait.task_id)))
        .collect())
}

pub(super) fn queued_task_ids(items: &[run_queue_item::Model]) -> HashSet<Id> {
    items.iter().filter(|item| !item.state.settled()).map(|item| item.task_id).collect()
}

pub(super) fn waiting_reason(waits: &[RunQueueWaitView]) -> String {
    let names: Vec<String> = waits
        .iter()
        .map(|wait| if wait.awaiting_review { format!("{} ({AWAITING_REVIEW_TEXT})", wait.anchor) } else { wait.anchor.clone() })
        .collect();
    format!("Waiting for {}.", names.join(", "))
}

pub(super) async fn view_of(tx: &Tx, queue: &run_queue::Model) -> Result<RunQueueView> {
    let all = ordered(tx).await?;
    let queued_tasks = queued_task_ids(&all);
    let mut items = Vec::new();
    for item in &all {
        items.push(item_view(tx, item, &queued_tasks).await?);
    }
    Ok(RunQueueView::of(queue, items))
}
