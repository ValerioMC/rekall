//! The square PNG a project shows as its icon, already normalised by the server (see
//! `ProjectIconService`). One row per project; absent means the project wears its icon key.

use rekall_common::{Id, Instant};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "project_icon_image")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub project_id: Id,
    pub png: Vec<u8>,
    /// Also the image's version: it changes with every upload, so a URL carrying it is immutable.
    pub updated_at: Instant,
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
