//! Where the person was last in a project: one of its tasks, or the task list when `task_id` is empty.
//! A remembered task that is deleted leaves the project at its task list.

use rekall_kernel::Id;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "working_set_place")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub project_id: Id,
    pub task_id: Option<Id>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "crate::project::Entity", from = "Column::ProjectId", to = "crate::project::Column::Id", on_delete = "Cascade")]
    Project,
}

impl Related<crate::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
