use rekall_kernel::Id;
use rekall_store::TaskStatus;

use crate::claude::Remaining;

/// One task another task depends on, with where its work stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskWait {
    pub task_id: Id,
    pub anchor: String,
    pub title: String,
    pub status: TaskStatus,
    pub remaining: Remaining,
}

/// How a claimed dependency is told to a person.
pub const AWAITING_REVIEW_TEXT: &str = "claimed, waiting for your review";

impl TaskWait {
    /// Accepted work, or a task a person set to `DONE`.
    pub fn is_met(&self) -> bool {
        self.status == TaskStatus::Done || self.remaining == Remaining::Accepted
    }

    /// Claimed work only waits for the review that accepts it.
    pub fn awaits_review(&self) -> bool {
        !self.is_met() && self.remaining == Remaining::Claimed
    }

    pub fn state_text(&self) -> &'static str {
        if self.is_met() {
            "accepted"
        } else if self.awaits_review() {
            AWAITING_REVIEW_TEXT
        } else {
            "not accepted"
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/dependency/model/task_wait_tests.rs"]
mod tests;
