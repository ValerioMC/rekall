//! One library a project declares, at the version it resolves to. A session reads the project's
//! manifests and writes the whole list at once; nothing edits one row.

use rekall_common::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;
use crate::Ecosystem;

pub const NAME_MAX_CHARACTERS: usize = 300;
pub const VERSION_MAX_CHARACTERS: usize = 100;
pub const MANIFEST_MAX_CHARACTERS: usize = 500;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "project_dependency")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub project_id: Id,
    pub ecosystem: Ecosystem,
    pub name: String,
    pub version: String,
    /// The file that declares it, relative to the project's folder.
    pub manifest: String,
    pub created_at: Instant,
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

impl Model {
    pub fn validate(&self) -> Result<(), RekallError> {
        Violations::new("ProjectDependency")
            .not_blank("name", Some(&self.name))
            .max_length("name", Some(&self.name), NAME_MAX_CHARACTERS)
            .not_blank("version", Some(&self.version))
            .max_length("version", Some(&self.version), VERSION_MAX_CHARACTERS)
            .not_blank("manifest", Some(&self.manifest))
            .max_length("manifest", Some(&self.manifest), MANIFEST_MAX_CHARACTERS)
            .finish()
    }

    /// Whether this row is the package an exposure names.
    pub fn is_package(&self, ecosystem: Ecosystem, name: &str, version: &str) -> bool {
        self.ecosystem == ecosystem && self.name == name && self.version == version
    }
}
