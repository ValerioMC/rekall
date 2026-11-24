//! A task the person keeps open. `position` is its place among the open tasks, counted from zero;
//! the set is kept without gaps.

use rekall_kernel::{Id, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "working_set_task")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub task_id: Id,
    pub position: i32,
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
    pub fn validate(&self) -> Result<(), RekallError> {
        Violations::new("WorkingSetTask").in_range("position", self.position, 0, i32::MAX).finish()
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/workingset/entity/working_set_task_tests.rs"]
mod tests;
