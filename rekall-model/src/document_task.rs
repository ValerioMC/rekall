//! The link between a note and a task. `position` is the note's place on that task
//! (`@OrderColumn` on `Task.documents`): dense from zero, per task, so one note can sit first on
//! one task and last on another.

use rekall_common::Id;
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
        belongs_to = "super::document::Entity",
        from = "Column::DocumentId",
        to = "super::document::Column::Id",
        on_delete = "Cascade"
    )]
    Document,
    #[sea_orm(
        belongs_to = "super::task::Entity",
        from = "Column::TaskId",
        to = "super::task::Column::Id",
        on_delete = "Cascade"
    )]
    Task,
}

impl Related<super::document::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Document.def()
    }
}

impl Related<super::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
