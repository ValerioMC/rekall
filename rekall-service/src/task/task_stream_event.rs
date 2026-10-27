use rekall_common::Id;
use serde::Serialize;

/// Emitted when a session creates a task or files one under an epic, so an open console reads its
/// task lists again without a reload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStreamEvent {
    pub task_id: Id,
    pub project_id: Id,
}

impl crate::StreamEvent for TaskStreamEvent {
    const FRAME: &'static str = "task-created";
}
