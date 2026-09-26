//! One line of a task's checklist, on its line `DRAFT -> OPEN -> RUNNING -> CLAIMED -> DONE`,
//! with the moment of each move beside the state.

use rekall_common::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::{Phase, Violations};
use crate::enums::TaskStepState;

pub const MAX_CHARACTERS: usize = 20_000;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "task_step")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub task_id: Id,
    pub title: String,
    pub body_markdown: Option<String>,
    pub state: TaskStepState,
    pub running_at: Option<Instant>,
    pub claimed_at: Option<Instant>,
    pub done_at: Option<Instant>,
    pub position: i32,
    pub created_at: Instant,
    pub updated_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::task::Entity",
        from = "Column::TaskId",
        to = "super::task::Column::Id",
        on_delete = "Cascade"
    )]
    Task,
}

impl Related<super::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// `new TaskStep(task, title, position)`: a draft, with nothing behind it.
    pub fn new(task_id: Id, title: String, position: i32, now: Instant) -> Self {
        Self {
            id: Id::random(),
            task_id,
            title,
            body_markdown: None,
            state: TaskStepState::Draft,
            running_at: None,
            claimed_at: None,
            done_at: None,
            position,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_done(&self) -> bool {
        self.state == TaskStepState::Done
    }

    /// When the work was finished: the claim first, then the console's tick. A wrapup written
    /// right after a claim already accounts for the work, and a later tick does not change it.
    pub fn completed_at(&self) -> Option<Instant> {
        self.claimed_at.or(self.done_at)
    }

    /// `TaskStep.markState`: the state and its moment are one fact. Reopening (or returning to
    /// draft) clears all three moments; claiming straight from open backfills `running_at`;
    /// marking the state the step is already in changes nothing.
    pub fn mark_state(&mut self, next: TaskStepState) {
        if self.state == next {
            return;
        }
        self.state = next;
        let now = Instant::now();
        match next {
            TaskStepState::Draft | TaskStepState::Open => {
                self.running_at = None;
                self.claimed_at = None;
                self.done_at = None;
            }
            TaskStepState::Running => {
                self.running_at = Some(now);
                self.claimed_at = None;
                self.done_at = None;
            }
            TaskStepState::Claimed => {
                if self.running_at.is_none() {
                    self.running_at = Some(now);
                }
                self.claimed_at = Some(now);
                self.done_at = None;
            }
            TaskStepState::Done => self.done_at = Some(now),
        }
    }

    pub fn validate(&self, phase: Phase) -> Result<(), RekallError> {
        Violations::new("TaskStep", phase)
            .not_blank("title", Some(&self.title))
            .size("title", Some(&self.title), 200)
            .size("bodyMarkdown", self.body_markdown.as_deref(), MAX_CHARACTERS)
            .finish()
    }
}

#[cfg(test)]
#[path = "../test/task_step_tests.rs"]
mod tests;
