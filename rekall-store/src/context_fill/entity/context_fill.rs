//! How much of its context window a chat held at the end of its last turn on a task, or on one of
//! its steps. One row per task with no step and one per step, overwritten each turn.

use rekall_kernel::{Id, Instant};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "context_fill")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub task_id: Id,
    /// Null for the task's own row, the fill of its last turn whichever step it ran on.
    pub step_id: Option<Id>,
    pub used_tokens: i64,
    pub window_tokens: i64,
    pub recorded_at: Instant,
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
