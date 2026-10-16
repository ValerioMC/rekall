use rekall_common::Instant;
use serde::Serialize;

use super::BackupReason;

/// One backup in the database folder's `backups/`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupFile {
    pub name: String,
    pub size_bytes: u64,
    pub created_at: Instant,
    pub reason: BackupReason,
}
