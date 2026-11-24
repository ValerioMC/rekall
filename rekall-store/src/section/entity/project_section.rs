//! A titled markdown section on a project, for what the blueprint does not hold, kept in one of the
//! project's section contexts. No task context ever loads one: a person or a session writes them,
//! and only the console and `rekall_read_sections` read them.

use rekall_kernel::{Id, Instant, RekallError};
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
    pub context_id: Id,
    pub title: String,
    pub body_markdown: String,
    /// Zero-based place in its context; a new section takes the next one.
    pub position: i32,
    pub created_at: Instant,
    pub updated_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "crate::project::Entity", from = "Column::ProjectId", to = "crate::project::Column::Id", on_delete = "Cascade")]
    Project,
    #[sea_orm(belongs_to = "crate::section_context::Entity", from = "Column::ContextId", to = "crate::section_context::Column::Id", on_delete = "NoAction")]
    Context,
}

impl Related<crate::project::Entity> for Entity {
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
