//! `task_id` waits for `depends_on_task_id` to be done. Both tasks belong to the same project;
//! the console service that writes the row enforces it.

use rekall_kernel::{Id, Instant};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "task_dependency")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub task_id: Id,
    pub depends_on_task_id: Id,
    pub created_at: Instant,
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
    pub fn new(task_id: Id, depends_on_task_id: Id, now: Instant) -> Self {
        Self { id: Id::random(), task_id, depends_on_task_id, created_at: now }
    }
}
