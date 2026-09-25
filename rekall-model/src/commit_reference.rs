//! A commit of the project's folder logged against a task, or one of its steps, with the diff it
//! introduced so the change can be reread without a checkout.

use rekall_common::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::{Phase, Violations};

pub const HASH_MAX: usize = 40;
pub const COMMENT_MAX: usize = 200;
pub const DIFF_MAX: usize = 200_000;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "commit_reference")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Id,
    pub task_id: Id,
    pub step_id: Option<Id>,
    pub commit_hash: String,
    pub comment: String,
    /// Null for a commit logged before the column existed, or when git could not produce one.
    #[sea_orm(column_type = "Text", nullable)]
    pub diff: Option<String>,
    /// Whether this commit rides along with `rekall_context`. Off when logged; the console flips
    /// it, one row at a time.
    pub in_context: bool,
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
    #[sea_orm(
        belongs_to = "super::task_step::Entity",
        from = "Column::StepId",
        to = "super::task_step::Column::Id",
        on_delete = "Cascade"
    )]
    TaskStep,
}

impl Related<super::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl Related<super::task_step::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TaskStep.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn validate(&self) -> Result<(), RekallError> {
        Violations::new("CommitReference", Phase::Persist)
            .column("commit_hash", Some(&self.commit_hash), HASH_MAX)
            .column("comment", Some(&self.comment), COMMENT_MAX)
            .finish()
    }
}
