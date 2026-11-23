use rekall_kernel::Id;
use serde::Serialize;

/// Emitted when a decision of a task is recorded, changed or removed, so an open console reloads the task's decisions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionStreamEvent {
    pub task_id: Id,
    pub key: String,
}

impl crate::StreamEvent for DecisionStreamEvent {
    const FRAME: &'static str = "decision";
}
