use rekall_kernel::{Id, Instant};
use rekall_store::{project, run_queue_item, task, RunQueueItemState};
use serde::Serialize;

use crate::claude::StepProgress;

/// One queued task, with enough of the task to name it without a second request, and its
/// released steps with the one being worked (none for a task with no checklist).
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
    pub steps: Vec<StepProgress>,
}

impl RunQueueItemView {
    pub fn of(item: &run_queue_item::Model, task: &task::Model, project: &project::Model, steps: Vec<StepProgress>) -> Self {
        Self {
            id: item.id,
            task_id: task.id,
            task_title: task.title.clone(),
            task_label: task.label.clone(),
            project_label: project.label.clone(),
            anchor: task.anchor(&project.label),
            position: item.position,
            state: item.state,
            detail: item.detail.clone(),
            started_at: item.started_at,
            finished_at: item.finished_at,
            steps,
        }
    }
}
