use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(
    /// Where a step is on its line, and, for a task with no checklist, where the task's own review
    /// line is.
    TaskStepState {
        Draft = "DRAFT",
        Open = "OPEN",
        Running = "RUNNING",
        Claimed = "CLAIMED",
        Done = "DONE",
    }
);

impl TaskStepState {
    pub fn complete(&self) -> bool {
        matches!(self, Self::Claimed | Self::Done)
    }

    pub fn running(&self) -> bool {
        *self == Self::Running
    }

    pub fn draft(&self) -> bool {
        *self == Self::Draft
    }

    pub fn reachable_by_session(&self) -> bool {
        !matches!(self, Self::Draft | Self::Done)
    }
}

#[cfg(test)]
#[path = "../../test/enums/task_step_state_tests.rs"]
mod tests;
