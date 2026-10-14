use rekall_common::Id;
use serde::Serialize;

/// Emitted when a session creates a task, so an open console lists it without a reload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStreamEvent {
    pub task_id: Id,
    pub project_id: Id,
}
