//! The link between a task and its issue in the project's task manager: the issue's key and URL once
//! it exists, the fingerprint of the content last written to it, and why the last push failed until one works. A task that was never pushed has no row.

use rekall_kernel::{Id, Instant, RekallError};
use sea_orm::entity::prelude::*;

use crate::constraints::Violations;

pub const ISSUE_KEY_MAX_CHARACTERS: usize = 200;
pub const ISSUE_URL_MAX_CHARACTERS: usize = 1_000;
pub const SYNC_ERROR_MAX_CHARACTERS: usize = 1_000;
pub const SYNCED_FINGERPRINT_MAX_CHARACTERS: usize = 16;
pub const ISSUE_TITLE_MAX_CHARACTERS: usize = 200;
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "task_tracker_link")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub task_id: Id,
    pub issue_key: Option<String>,
    pub issue_url: Option<String>,
    pub sync_error: Option<String>,
    pub synced_fingerprint: Option<String>,
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

/// A task's title as its issue carries it.
pub fn issue_title(title: &str) -> String {
    title.chars().take(ISSUE_TITLE_MAX_CHARACTERS).collect()
}

/// FNV-1a over the title and the description, in hex. It is stored, so it must not change from
/// one Rust release to the next, which the standard hasher does not promise.
pub fn content_fingerprint(title: &str, description: &str) -> String {
    let bytes = title.bytes().chain(std::iter::once(0)).chain(description.bytes());
    let hash = bytes.fold(FNV_OFFSET_BASIS, |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME));
    format!("{hash:016x}")
}

impl Model {
    /// Whether what was last written to the issue is the task's title and description as they are now.
    pub fn holds_content_of(&self, task: &crate::task::Model) -> bool {
        let current = content_fingerprint(&issue_title(&task.title), task.description.as_deref().unwrap_or_default());
        self.synced_fingerprint.as_deref() == Some(current.as_str())
    }

    pub fn validate(&self) -> Result<(), RekallError> {
        Violations::new("TaskTrackerLink")
            .max_length("issueKey", self.issue_key.as_deref(), ISSUE_KEY_MAX_CHARACTERS)
            .max_length("issueUrl", self.issue_url.as_deref(), ISSUE_URL_MAX_CHARACTERS)
            .max_length("syncError", self.sync_error.as_deref(), SYNC_ERROR_MAX_CHARACTERS)
            .max_length("syncedFingerprint", self.synced_fingerprint.as_deref(), SYNCED_FINGERPRINT_MAX_CHARACTERS)
            .finish()
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/tracker/entity/task_tracker_link_tests.rs"]
mod tests;
