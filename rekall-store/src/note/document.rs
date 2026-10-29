//! A note: freeform markdown attached to one or more tasks through `document_task`.

use rekall_kernel::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;
use crate::enums::DocumentContextMode;
use crate::note_scope::NoteScope;

/// A note's anchor is `note:` and the first eight characters of its id.
pub const ANCHOR_ID_LENGTH: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "document")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub title: String,
    pub kind: String,
    pub body_markdown: String,
    pub source_path: Option<String>,
    pub context_mode: DocumentContextMode,
    /// Set for a note owned by one company; see `NoteScope`.
    pub scope_company_id: Option<Id>,
    /// Set for a note owned by one project; never alongside `scope_company_id`.
    pub scope_project_id: Option<Id>,
    pub created_at: Instant,
    pub updated_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "crate::document_task::Entity")]
    DocumentTask,
}

impl Related<crate::document_task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DocumentTask.def()
    }
}

impl Related<crate::task::Entity> for Entity {
    fn to() -> RelationDef {
        crate::document_task::Relation::Task.def()
    }

    fn via() -> Option<RelationDef> {
        Some(crate::document_task::Relation::Document.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn scope(&self) -> NoteScope {
        NoteScope::of_columns(self.scope_company_id, self.scope_project_id)
    }

    pub fn set_scope(&mut self, scope: NoteScope) {
        (self.scope_company_id, self.scope_project_id) = scope.columns();
    }

    /// The short handle a session loads a reference note by.
    pub fn anchor(&self) -> String {
        format!("note:{}", &self.id.to_string()[..ANCHOR_ID_LENGTH])
    }

    pub fn validate(&self) -> Result<(), RekallError> {
        Violations::new("Document")
            .not_blank("title", Some(&self.title))
            .max_length("title", Some(&self.title), 255)
            .not_blank("kind", Some(&self.kind))
            .max_length("kind", Some(&self.kind), 40)
            .max_length("bodyMarkdown", Some(&self.body_markdown), 100_000)
            .max_length("sourcePath", self.source_path.as_deref(), 500)
            .finish()
    }
}
