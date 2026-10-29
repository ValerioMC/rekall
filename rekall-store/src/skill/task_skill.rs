//! A Claude skill assigned to a task, or to one of its steps when `step_id` is set. A task's own
//! skills apply to every step of it; a step's apply to that step alone.

use rekall_common::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::skill_name;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "task_skill")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub task_id: Id,
    pub step_id: Option<Id>,
    pub skill_name: String,
    pub position: i32,
    pub created_at: Instant,
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
    pub fn validate(&self) -> Result<(), RekallError> {
        skill_name::validate(&self.skill_name)
    }
}
