use rekall_common::{Id, Instant};
use rekall_model::task;
use rekall_model::TaskStepState;
use serde::Serialize;

/// The review line as the console reads it. `review_active` is false once the task has a
/// checklist, and the rest then means nothing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskReviewView {
    pub task_id: Id,
    pub review_state: TaskStepState,
    pub review_active: bool,
    pub claimed_at: Option<Instant>,
    pub accepted_at: Option<Instant>,
    pub review_note: Option<String>,
}

impl TaskReviewView {
    pub fn of(task: &task::Model, review_active: bool) -> Self {
        Self {
            task_id: task.id,
            review_state: task.review_state,
            review_active,
            claimed_at: task.claimed_at,
            accepted_at: task.accepted_at,
            review_note: task.review_note.clone(),
        }
    }
}
