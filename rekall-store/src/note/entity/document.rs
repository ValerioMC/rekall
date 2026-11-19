//! A note: freeform markdown that belongs to one task and goes with it. `position` is its place
//! on that task, dense from zero.

use rekall_kernel::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;
use crate::enums::DocumentContextMode;

/// A note's anchor is `note:` and the first eight characters of its id.
pub const ANCHOR_ID_LENGTH: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "document")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub task_id: Id,
    pub position: i32,
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

impl Model {
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
