//! One entry into a step of a flow, numbered in the order the flow moved. The open visit (no
//! `left_at`) is where the flow stands; `outcome` is the route it left by, `report` what the
//! session saw there.

use rekall_kernel::{Id, Instant};
use sea_orm::entity::prelude::*;

/// What a session may report when it leaves a step: enough for a failing job's log excerpt.
pub const REPORT_MAX_CHARACTERS: usize = 4_000;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "step_visit")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub task_id: Id,
    pub step_id: Id,
    /// One-based and dense per task: the move number the console prints.
    pub ordinal: i32,
    pub entered_at: Instant,
    pub left_at: Option<Instant>,
    pub outcome: Option<String>,
    pub report: Option<String>,
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
    #[sea_orm(
        belongs_to = "crate::task_step::Entity",
        from = "Column::StepId",
        to = "crate::task_step::Column::Id",
        on_delete = "Cascade"
    )]
    TaskStep,
}

impl Related<crate::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl Related<crate::task_step::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TaskStep.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn enter(task_id: Id, step_id: Id, ordinal: i32, now: Instant) -> Self {
        Self { id: Id::random(), task_id, step_id, ordinal, entered_at: now, left_at: None, outcome: None, report: None }
    }

    pub fn is_open(&self) -> bool {
        self.left_at.is_none()
    }

    /// Closing a visit twice keeps the first close: the route it left by is history.
    pub fn leave(&mut self, outcome: Option<String>, report: Option<String>, now: Instant) {
        if !self.is_open() {
            return;
        }
        self.left_at = Some(now);
        self.outcome = outcome;
        self.report = report;
    }
}

#[cfg(test)]
#[path = "../../tests/unit/flow/step_visit_tests.rs"]
mod tests;
