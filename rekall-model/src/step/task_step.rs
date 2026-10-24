//! One line of a task's checklist, on its line `DRAFT -> OPEN -> RUNNING -> CLAIMED -> DONE`,
//! with the moment of each move beside the state.

use rekall_common::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;
use crate::enums::TaskStepState;
use crate::step_pass::StepPass;

pub const MAX_CHARACTERS: usize = 20_000;

pub const TITLE_MAX_CHARACTERS: usize = 200;

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
    /// The earlier passes, oldest first, as a JSON array of `StepPass`.
    pub passes_json: String,
    /// Where the person dragged the step on the flow canvas; `None` leaves it to the automatic layout.
    pub flow_x: Option<i32>,
    pub flow_y: Option<i32>,
    /// How many times a flow may enter the step; `None` sets no limit.
    pub max_visits: Option<i32>,
    /// The company connection the step reaches, carried over from the deployment graph it came from.
    pub connection_id: Option<Id>,
    pub created_at: Instant,
    pub updated_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::task::Entity",
        from = "Column::TaskId",
        to = "crate::task::Column::Id",
        on_delete = "Cascade"
    )]
    Task,
}

impl Related<crate::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// A draft step, with nothing behind it.
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
            passes_json: "[]".to_string(),
            flow_x: None,
            flow_y: None,
            max_visits: None,
            connection_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Whether the flow may enter the step again after `entered` visits.
    pub fn admits_visit(&self, entered: usize) -> bool {
        self.max_visits.is_none_or(|max| entered < max as usize)
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

    pub fn passes(&self) -> Result<Vec<StepPass>, RekallError> {
        serde_json::from_str(&self.passes_json)
            .map_err(|error| RekallError::illegal(format!("Step '{}' holds unreadable passes: {error}", self.title)))
    }

    /// A claim sent back for another pass: the detail that pass worked from is kept as a pass, and
    /// the detail starts empty, ready for the feedback. Only a claimed step can be sent back.
    pub fn send_back(&mut self, now: Instant) -> Result<(), RekallError> {
        if self.state != TaskStepState::Claimed {
            return Err(RekallError::illegal(format!(
                "'{}' is {}; only a claimed step is sent back for another pass.",
                self.title,
                self.state.name().to_lowercase()
            )));
        }
        let mut passes = self.passes()?;
        passes.push(StepPass { detail_markdown: self.body_markdown.take(), sent_back_at: now });
        self.passes_json = serde_json::to_string(&passes)
            .map_err(|error| RekallError::illegal(format!("Could not keep the pass: {error}")))?;
        self.mark_state(TaskStepState::Open);
        Ok(())
    }

    pub fn validate(&self) -> Result<(), RekallError> {
        Violations::new("TaskStep")
            .not_blank("title", Some(&self.title))
            .max_length("title", Some(&self.title), TITLE_MAX_CHARACTERS)
            .max_length("bodyMarkdown", self.body_markdown.as_deref(), MAX_CHARACTERS)
            .finish()
    }
}

#[cfg(test)]
#[path = "../../tests/unit/step/task_step_tests.rs"]
mod tests;
