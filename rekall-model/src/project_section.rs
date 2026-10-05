//! A titled markdown section on a project, for what the blueprint does not hold. Sections are
//! never part of any context: a person or a session writes them, and only the console reads them.

use rekall_common::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;

pub const TITLE_MAX_CHARACTERS: usize = 200;
pub const BODY_MAX_CHARACTERS: usize = 100_000;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "project_section")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub project_id: Id,
    pub title: String,
    pub body_markdown: String,
    /// Zero-based place in the project's list; a new section takes the next one.
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
        Violations::new("ProjectSection")
            .not_blank("title", Some(&self.title))
            .max_length("title", Some(&self.title), TITLE_MAX_CHARACTERS)
            .max_length("bodyMarkdown", Some(&self.body_markdown), BODY_MAX_CHARACTERS)
            .finish()
    }
}
