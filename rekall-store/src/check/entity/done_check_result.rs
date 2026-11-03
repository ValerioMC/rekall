//! What one run of a check found, kept with the step (or the task with no steps) it was run for.

use rekall_kernel::{Id, Instant};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "done_check_result")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub check_id: Id,
    pub task_id: Id,
    /// Absent for a task with no steps.
    pub step_id: Option<Id>,
    pub passed: bool,
    /// Absent when the command timed out.
    pub exit_code: Option<i32>,
    pub output_tail: String,
    pub ran_at: Instant,
    pub duration_ms: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "crate::done_check::Entity", from = "Column::CheckId", to = "crate::done_check::Column::Id", on_delete = "Cascade")]
    DoneCheck,
    #[sea_orm(belongs_to = "crate::task::Entity", from = "Column::TaskId", to = "crate::task::Column::Id", on_delete = "Cascade")]
    Task,
    #[sea_orm(belongs_to = "crate::task_step::Entity", from = "Column::StepId", to = "crate::task_step::Column::Id", on_delete = "Cascade")]
    TaskStep,
}

impl Related<crate::done_check::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DoneCheck.def()
    }
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
