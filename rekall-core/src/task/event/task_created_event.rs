use rekall_kernel::Id;
use serde::Serialize;

/// Emitted when a task is created: the project's task manager is told its title and nothing more,
/// since the description is written there only when the person asks. A task that arrives from the
/// task manager does not emit it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskCreatedEvent {
    pub task_id: Id,
}

impl crate::StreamEvent for TaskCreatedEvent {
    const FRAME: &'static str = "task-created";
}
