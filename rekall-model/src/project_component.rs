//! A folder of a project that is a part of it in its own right: a module, a service, an app.
//! Components travel into every context that loads the project, so a session knows where each
//! part lives and what it is.

use rekall_common::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;

pub const NAME_MAX_CHARACTERS: usize = 200;
pub const FOLDER_MAX_CHARACTERS: usize = 1_000;
pub const DESCRIPTION_MAX_CHARACTERS: usize = 10_000;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "project_component")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub project_id: Id,
    pub name: String,
    pub folder: String,
    pub description: String,
    /// Zero-based place in the project's list; a new component takes the next one.
    pub position: i32,
    pub created_at: Instant,
    pub updated_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::project::Entity", from = "Column::ProjectId", to = "super::project::Column::Id", on_delete = "Cascade")]
    Project,
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn validate(&self) -> Result<(), RekallError> {
        Violations::new("ProjectComponent")
            .not_blank("name", Some(&self.name))
            .max_length("name", Some(&self.name), NAME_MAX_CHARACTERS)
            .not_blank("folder", Some(&self.folder))
            .max_length("folder", Some(&self.folder), FOLDER_MAX_CHARACTERS)
            .max_length("description", Some(&self.description), DESCRIPTION_MAX_CHARACTERS)
            .finish()
    }
}
