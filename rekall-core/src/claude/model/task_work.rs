use rekall_kernel::{Id, Instant};

use crate::claude::Remaining;

/// `settled_steps` counts the claimed and done steps, so a rise between two reads is a claim;
/// `running_step_ids` are the steps a session has marked running; `last_settled_at` is the latest
/// claim or acceptance; `next_opens_session` says the step to work next starts a session of its own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskWork {
    pub remaining: Remaining,
    pub settled_steps: usize,
    pub running_step_ids: Vec<Id>,
    pub last_settled_at: Option<Instant>,
    pub next_opens_session: bool,
}

impl TaskWork {
    pub fn has_work(&self) -> bool {
        self.remaining == Remaining::Open
    }

    /// Whether the session that started at `started_at` has claimed the last step of its group:
    /// work is left, none of it is running, and the step next opens a session of its own.
    pub fn session_is_spent(&self, started_at: &Instant) -> bool {
        self.has_work()
            && self.next_opens_session
            && self.running_step_ids.is_empty()
            && self.last_settled_at.is_some_and(|settled| settled.is_after(started_at))
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/claude/model/task_work_tests.rs"]
mod tests;
