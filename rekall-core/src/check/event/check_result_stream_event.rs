use rekall_kernel::Id;
use serde::Serialize;

/// Emitted when a run of a check is kept, so an open console reads the task's results again
/// while the session is still working.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckResultStreamEvent {
    pub task_id: Id,
}

impl crate::StreamEvent for CheckResultStreamEvent {
    const FRAME: &'static str = "task-check-result";
}
