//! The pull request (GitLab: merge request) opened for a task's branch: its number and page once it
//! exists, when it was merged, and why the last attempt to open or merge it failed. A task that never reached a claim
//! has no row.

use rekall_kernel::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;

pub const PULL_REQUEST_URL_MAX_CHARACTERS: usize = 1_000;
pub const FAILURE_MAX_CHARACTERS: usize = 1_000;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "task_pull_request")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub task_id: Id,
    pub number: Option<i64>,
    pub url: Option<String>,
    pub failure: Option<String>,
    pub merged_at: Option<Instant>,
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
    pub fn validate(&self) -> Result<(), RekallError> {
        Violations::new("TaskPullRequest")
            .max_length("url", self.url.as_deref(), PULL_REQUEST_URL_MAX_CHARACTERS)
            .max_length("failure", self.failure.as_deref(), FAILURE_MAX_CHARACTERS)
            .finish()
    }
}
