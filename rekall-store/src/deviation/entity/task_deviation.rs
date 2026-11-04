//! What a task did differently from its plan at one step: what was planned, what was built, why. The
//! step is named by its number, which is its place in the checklist when the deviation was written.

use rekall_kernel::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;

pub const BODY_MAX_CHARACTERS: usize = 100_000;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "task_deviation")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub task_id: Id,
    pub step_number: i32,
    pub body_markdown: String,
    pub created_at: Instant,
    pub updated_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "crate::task::Entity", from = "Column::TaskId", to = "crate::task::Column::Id", on_delete = "Cascade")]
    Task,
}

impl Related<crate::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn new(task_id: Id, step_number: i32, body_markdown: String, now: Instant) -> Self {
        Self { id: Id::random(), task_id, step_number, body_markdown, created_at: now, updated_at: now }
    }

    pub fn validate(&self) -> Result<(), RekallError> {
        Violations::new("Deviation")
            .in_range("stepNumber", self.step_number, 1, i32::MAX)
            .not_blank("bodyMarkdown", Some(&self.body_markdown))
            .max_length("bodyMarkdown", Some(&self.body_markdown), BODY_MAX_CHARACTERS)
            .finish()
    }
}
