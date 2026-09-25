//! The enumerations, stored by name (`@Enumerated(EnumType.STRING)`) and written on the wire by
//! name, as Jackson wrote a Java enum.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

macro_rules! string_enum {
    ($(#[$meta:meta])* $name:ident { $($variant:ident = $text:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
        #[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
        pub enum $name {
            $(
                #[sea_orm(string_value = $text)]
                #[serde(rename = $text)]
                $variant,
            )+
        }

        impl $name {
            /// `Enum.name()`.
            pub fn name(&self) -> &'static str {
                match self { $(Self::$variant => $text,)+ }
            }

            /// `Enum.ordinal()`.
            pub fn ordinal(&self) -> usize {
                <Self as sea_orm::Iterable>::iter().position(|v| v == *self).unwrap_or(0)
            }

            /// `Enum.valueOf(name)`.
            pub fn value_of(name: &str) -> Option<Self> {
                match name { $($text => Some(Self::$variant),)+ _ => None }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.name())
            }
        }
    };
}

string_enum!(ProjectStatus { Active = "ACTIVE", Paused = "PAUSED", Done = "DONE" });

string_enum!(TaskStatus { Todo = "TODO", InProgress = "IN_PROGRESS", Blocked = "BLOCKED", Done = "DONE" });

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

string_enum!(WrapupAuthor { Claude = "CLAUDE", Hand = "HAND" });

string_enum!(
    /// How a note travels in a task's context: in full under every task it is on, or as its
    /// title, a line of it and an anchor a session loads only when the work needs it.
    DocumentContextMode { Full = "FULL", Reference = "REFERENCE" }
);

string_enum!(
    /// Which text of a task a revision keeps an earlier version of.
    RevisionKind { Wrapup = "WRAPUP", Description = "DESCRIPTION" }
);

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
mod tests {
    use super::*;

    #[test]
    fn a_draft_is_out_of_a_sessions_reach_and_so_is_done() {
        assert!(!TaskStepState::Draft.reachable_by_session());
        assert!(TaskStepState::Open.reachable_by_session());
        assert!(TaskStepState::Running.reachable_by_session());
        assert!(TaskStepState::Claimed.reachable_by_session());
        assert!(!TaskStepState::Done.reachable_by_session());
    }

    #[test]
    fn ordinals_follow_declaration_order() {
        assert_eq!(TaskStepState::Draft.ordinal(), 0);
        assert_eq!(TaskStepState::Done.ordinal(), 4);
        assert_eq!(RunQueueItemState::value_of("SKIPPED"), Some(RunQueueItemState::Skipped));
    }
}
