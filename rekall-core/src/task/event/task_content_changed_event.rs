use rekall_kernel::Id;
use serde::Serialize;

/// Emitted when a task is created or its title or description changes: what the project's task
/// manager is told about. A task that arrives from the task manager does not emit it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskContentChangedEvent {
    pub task_id: Id,
}

impl crate::StreamEvent for TaskContentChangedEvent {
    const FRAME: &'static str = "task-content-changed";
}
