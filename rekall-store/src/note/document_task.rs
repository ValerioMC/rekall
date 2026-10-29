//! The link between a note and a task. `position` is the note's place on that task: dense from
//! zero, per task, so one note can sit first on one task and last on another.

use rekall_kernel::Id;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "document_task")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub document_id: Id,
    #[sea_orm(primary_key, auto_increment = false)]
    pub task_id: Id,
    pub position: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::document::Entity",
        from = "Column::DocumentId",
        to = "crate::document::Column::Id",
        on_delete = "Cascade"
    )]
    Document,
    #[sea_orm(
        belongs_to = "crate::task::Entity",
        from = "Column::TaskId",
        to = "crate::task::Column::Id",
        on_delete = "Cascade"
    )]
    Task,
}

impl Related<crate::document::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Document.def()
    }
}

impl Related<crate::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
