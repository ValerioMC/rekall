use rekall_kernel::Id;
use serde::Serialize;

/// Emitted when a thread changes, so an open console refetches it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackStreamEvent {
    pub task_id: Id,
}

impl crate::StreamEvent for FeedbackStreamEvent {
    const FRAME: &'static str = "feedback";
}
