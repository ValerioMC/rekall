//! The tokens one Claude Code session spent on one task, or on one of its steps. A session that
//! worked several steps has a row per step, plus one for the task when it did work outside them.
//! A row also has a day, so a task's spend can be told day by day. The rows of a session are
//! rewritten whole each time its transcript is read again.

use rekall_common::{Id, Instant};
use sea_orm::entity::prelude::*;

pub const SESSION_ID_MAX: usize = 100;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "token_usage")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub session_id: String,
    pub task_id: Id,
    /// Null for work on the task outside any step, and for a step deleted since.
    pub step_id: Option<Id>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cache_read_tokens: i64,
    pub recorded_at: Instant,
    /// Midnight UTC of the day the tokens were generated, not of the read that found them.
    pub spent_on: Instant,
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
        on_delete = "SetNull"
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
