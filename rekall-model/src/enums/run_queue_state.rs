use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(
    /// Where the run queue is on its line. `Idle` runs nothing; `Scheduled` waits for its start
    /// time; `Running` has a session on the head item, or is about to open one; `Holding` has
    /// reached the usage ceiling and waits for the window to reset.
    RunQueueState { Idle = "IDLE", Scheduled = "SCHEDULED", Running = "RUNNING", Holding = "HOLDING" }
);

impl RunQueueState {
    /// True while the queue has been started and not stopped or run dry.
    pub fn armed(&self) -> bool {
        *self != Self::Idle
    }
}
