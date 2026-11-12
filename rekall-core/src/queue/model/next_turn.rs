use rekall_kernel::Id;

use crate::queue::RunQueueItemView;

/// A waiting item and why it cannot start yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockedItem {
    pub item_id: Id,
    pub reason: String,
}

/// What the queue has for the worker: the first item that may start, or nothing startable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NextTurn {
    Ready(Box<RunQueueItemView>),
    /// Items wait, and every one waits for a task that is not accepted.
    Blocked(Vec<BlockedItem>),
    Empty,
}
