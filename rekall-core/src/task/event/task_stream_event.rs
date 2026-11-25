use rekall_kernel::Id;
use serde::Serialize;

/// Emitted when a session creates a task, files one under an epic or changes its description, so an
/// open console reads its task lists again without a reload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStreamEvent {
    pub task_id: Id,
    pub project_id: Id,
}

impl crate::StreamEvent for TaskStreamEvent {
    const FRAME: &'static str = "task-created";
}
