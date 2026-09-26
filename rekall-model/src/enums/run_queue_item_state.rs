use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(
    /// One queued task's outcome. `Queued` waits its turn; `Running` has a session on it; the
    /// other three are where it stopped.
    RunQueueItemState {
        Queued = "QUEUED",
        Running = "RUNNING",
        Finished = "FINISHED",
        Skipped = "SKIPPED",
        Failed = "FAILED",
    }
);

impl RunQueueItemState {
    /// True once the queue is finished with the item, whatever the outcome.
    pub fn settled(&self) -> bool {
        matches!(self, Self::Finished | Self::Skipped | Self::Failed)
    }
}

#[cfg(test)]
#[path = "../../test/enums/run_queue_item_state_tests.rs"]
mod tests;
