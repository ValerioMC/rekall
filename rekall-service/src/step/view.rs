use rekall_common::{Id, Instant};
use rekall_model::task_step;
use rekall_model::TaskStepState;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStepView {
    pub id: Id,
    pub task_id: Id,
    pub title: String,
    pub body_markdown: Option<String>,
    pub state: TaskStepState,
    pub done: bool,
    pub running_at: Option<Instant>,
    pub claimed_at: Option<Instant>,
    pub done_at: Option<Instant>,
    pub position: i32,
    pub created_at: Instant,
    pub updated_at: Instant,
}

impl TaskStepView {
    pub fn of(step: &task_step::Model) -> Self {
        Self {
            id: step.id,
            task_id: step.task_id,
            title: step.title.clone(),
            body_markdown: step.body_markdown.clone(),
            state: step.state,
            done: step.is_done(),
            running_at: step.running_at,
            claimed_at: step.claimed_at,
            done_at: step.done_at,
            position: step.position,
            created_at: step.created_at,
            updated_at: step.updated_at,
        }
    }

    pub fn completed_at(&self) -> Option<Instant> {
        self.claimed_at.or(self.done_at)
    }
}

/// A task's whole checklist after a write, from the console or from MCP alike.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepStreamEvent {
    pub task_id: Id,
    pub steps: Vec<TaskStepView>,
}
