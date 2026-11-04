use rekall_kernel::{Id, Instant};
use rekall_store::task_deviation;

/// One deviation of a task, by the step it refers to; the step's title is that of the checklist
/// today, absent when the checklist no longer has a step at that number.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskDeviationView {
    pub id: Id,
    pub task_id: Id,
    pub step_number: i32,
    pub step_title: Option<String>,
    pub body_markdown: String,
    pub created_at: Instant,
    pub updated_at: Instant,
}

impl TaskDeviationView {
    pub fn of(deviation: &task_deviation::Model, step_title: Option<String>) -> Self {
        Self {
            id: deviation.id,
            task_id: deviation.task_id,
            step_number: deviation.step_number,
            step_title,
            body_markdown: deviation.body_markdown.clone(),
            created_at: deviation.created_at,
            updated_at: deviation.updated_at,
        }
    }
}
