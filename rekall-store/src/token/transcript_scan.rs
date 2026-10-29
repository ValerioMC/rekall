//! How much of a Claude Code session's transcript has been read into `token_usage`. A transcript
//! only grows, so one whose size has not moved since is not read again.

use rekall_common::Instant;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "transcript_scan")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub session_id: String,
    /// The bytes of the session's files, its subagents' included, when they were last read.
    pub scanned_bytes: i64,
    pub scanned_at: Instant,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
