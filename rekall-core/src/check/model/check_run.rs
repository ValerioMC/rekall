use std::time::Duration;

use super::check_outcome::CheckOutcome;

/// What a `CheckRunner` saw: how the command ended, the last lines it printed, how long it took.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckRun {
    pub outcome: CheckOutcome,
    pub output_tail: String,
    pub duration: Duration,
}
