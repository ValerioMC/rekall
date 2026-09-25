//! An earlier version of a task's wrapup or description, kept when something replaced or deleted
//! it. Immutable once written: restoring one writes it back as the current text.

use rekall_common::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::{Phase, Violations};
use crate::enums::{RevisionKind, WrapupAuthor};

pub const MAX_CHARACTERS: usize = 100_000;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "task_revision")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub task_id: Id,
    pub kind: RevisionKind,
    pub body_markdown: String,
    /// Who wrote the version kept here; known for a wrapup, absent for a description.
    pub written_by: Option<WrapupAuthor>,
    /// When the version kept here was written, when that is known.
    pub written_at: Option<Instant>,
    /// When it stopped being the current text.
    pub created_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::task::Entity",
        from = "Column::TaskId",
        to = "super::task::Column::Id",
        on_delete = "Cascade"
    )]
    Task,
}

impl Related<super::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn validate(&self) -> Result<(), RekallError> {
        Violations::new("TaskRevision", Phase::Persist)
            .column("body_markdown", Some(&self.body_markdown), MAX_CHARACTERS)
            .finish()
    }
}
