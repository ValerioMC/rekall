use rekall_kernel::Id;
use serde::Serialize;

/// Emitted when a session writes or rewrites a deviation, so an open console can pick it up.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviationStreamEvent {
    pub task_id: Id,
    pub step_number: i32,
}

impl crate::StreamEvent for DeviationStreamEvent {
    const FRAME: &'static str = "deviation";
}
