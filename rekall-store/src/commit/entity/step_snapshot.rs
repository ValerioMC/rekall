//! The moment a step, or a task with no steps, started running, for the working tree it found.
//! Its files are the ones already modified then; none means the tree was clean.

use rekall_kernel::{Id, Instant};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "step_snapshot")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub task_id: Id,
    /// Absent for the snapshot of a task that has no steps.
    pub step_id: Option<Id>,
    pub taken_at: Instant,
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
