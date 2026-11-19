//! One text a task's wrapup has had. `number` counts up per task, so the newest is the highest; at
//! most `KEPT_PER_TASK` stay. `commit_hash` is the project's `HEAD` when the text was written.

use rekall_kernel::{Id, Instant};
use sea_orm::entity::prelude::*;

use crate::enums::WrapupAuthor;

pub const KEPT_PER_TASK: usize = 20;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "wrapup_version")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub task_id: Id,
    pub number: i32,
    pub body_markdown: String,
    pub written_by: WrapupAuthor,
    pub written_at: Instant,
    pub commit_hash: Option<String>,
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
}

impl Related<crate::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
