use serde::Serialize;

use super::BackupFile;

/// What the settings panel shows: where backups go, which there are, and the last thing that failed.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupStatus {
    pub available: bool,
    pub folder: Option<String>,
    pub interval_hours: i64,
    pub keep: usize,
    pub backups: Vec<BackupFile>,
    pub last_failure: Option<String>,
}
