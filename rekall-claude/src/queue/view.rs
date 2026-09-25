use rekall_common::{Id, Instant};
use rekall_model::{project, run_queue, run_queue_item, task, RunQueueItemState, RunQueueState};
use serde::Serialize;

/// The run queue as the console draws it: its state and settings, and every item in order. The
/// same shape answers every call and rides the `run-queue` event.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunQueueView {
    pub state: RunQueueState,
    pub start_at: Option<Instant>,
    pub ceiling_percent: Option<i32>,
    pub skip_permissions: bool,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub hold_until: Option<Instant>,
    pub hold_reason: Option<String>,
    pub items: Vec<RunQueueItemView>,
    pub updated_at: Instant,
}

/// One queued task, with enough of the task to name it without a second request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunQueueItemView {
    pub id: Id,
    pub task_id: Id,
    pub task_title: String,
    pub task_label: String,
    pub project_label: String,
    pub anchor: String,
    pub position: i32,
    pub state: RunQueueItemState,
    pub detail: Option<String>,
    pub started_at: Option<Instant>,
    pub finished_at: Option<Instant>,
}

impl RunQueueItemView {
    pub fn of(item: &run_queue_item::Model, task: &task::Model, project: &project::Model) -> Self {
        Self {
            id: item.id,
            task_id: task.id,
            task_title: task.title.clone(),
            task_label: task.label.clone(),
            project_label: project.label.clone(),
            anchor: format!("project:{} task:{}", project.label, task.label),
            position: item.position,
            state: item.state,
            detail: item.detail.clone(),
            started_at: item.started_at,
            finished_at: item.finished_at,
        }
    }
}

impl RunQueueView {
    pub fn of(queue: &run_queue::Model, items: Vec<RunQueueItemView>) -> Self {
        Self {
            state: queue.state,
            start_at: queue.start_at,
            ceiling_percent: queue.ceiling_percent,
            skip_permissions: queue.skip_permissions,
            model: queue.model.clone(),
            effort: queue.effort.clone(),
            hold_until: queue.hold_until,
            hold_reason: queue.hold_reason.clone(),
            items,
            updated_at: queue.updated_at,
        }
    }
}
