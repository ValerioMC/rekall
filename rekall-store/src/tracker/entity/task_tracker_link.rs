//! The link between a task and its issue in the project's task manager: the issue's key and URL once
//! it exists, and why the last push failed until one works. A task that was never pushed has no row.

use rekall_kernel::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;

pub const ISSUE_KEY_MAX_CHARACTERS: usize = 200;
pub const ISSUE_URL_MAX_CHARACTERS: usize = 1_000;
pub const SYNC_ERROR_MAX_CHARACTERS: usize = 1_000;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "task_tracker_link")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub task_id: Id,
    pub issue_key: Option<String>,
    pub issue_url: Option<String>,
    pub sync_error: Option<String>,
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
        Violations::new("TaskTrackerLink")
            .max_length("issueKey", self.issue_key.as_deref(), ISSUE_KEY_MAX_CHARACTERS)
            .max_length("issueUrl", self.issue_url.as_deref(), ISSUE_URL_MAX_CHARACTERS)
            .max_length("syncError", self.sync_error.as_deref(), SYNC_ERROR_MAX_CHARACTERS)
            .finish()
    }
}
