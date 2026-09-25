//! A note: freeform markdown attached to one or more tasks through `document_task`.

use rekall_common::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::{Phase, Violations};
use crate::enums::DocumentContextMode;

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
    pub created_at: Instant,
    pub updated_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::document_task::Entity")]
    DocumentTask,
}

impl Related<super::document_task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DocumentTask.def()
    }
}

impl Related<super::task::Entity> for Entity {
    fn to() -> RelationDef {
        super::document_task::Relation::Task.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::document_task::Relation::Document.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// The short handle a session loads a reference note by.
    pub fn anchor(&self) -> String {
        format!("note:{}", &self.id.to_string()[..ANCHOR_ID_LENGTH])
    }

    pub fn validate(&self, phase: Phase) -> Result<(), RekallError> {
        Violations::new("Document", phase)
            .not_blank("title", Some(&self.title))
            .size("title", Some(&self.title), 255)
            .not_blank("kind", Some(&self.kind))
            .size("kind", Some(&self.kind), 40)
            .column("body_markdown", Some(&self.body_markdown), 100_000)
            .column("source_path", self.source_path.as_deref(), 500)
            .finish()
    }
}
