use serde::Serialize;

use crate::dependency::TaskWait;

/// A task the queued one still waits for. `in_queue` is false when that task is not on the list,
/// so nothing in the queue will ever make it ready.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunQueueWaitView {
    pub anchor: String,
    pub title: String,
    pub awaiting_review: bool,
    pub in_queue: bool,
}

impl RunQueueWaitView {
    pub fn of(wait: &TaskWait, in_queue: bool) -> Self {
        Self { anchor: wait.anchor.clone(), title: wait.title.clone(), awaiting_review: wait.awaits_review(), in_queue }
    }
}
