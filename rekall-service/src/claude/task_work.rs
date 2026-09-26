use rekall_common::Id;

use super::Remaining;

/// `settled_steps` counts the claimed and done steps, so a rise between two reads is a claim;
/// `running_step_ids` are the steps a session has marked running.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskWork {
    pub remaining: Remaining,
    pub settled_steps: usize,
    pub running_step_ids: Vec<Id>,
}

impl TaskWork {
    pub fn has_work(&self) -> bool {
        self.remaining == Remaining::Open
    }
}
