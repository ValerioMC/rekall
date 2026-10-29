use serde::Serialize;

use super::BackupFile;

/// A restore that was accepted: the application is restarting, and this backup holds what was there.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RestoreStarted {
    pub(super) restarting: bool,
    pub(super) previous_state: BackupFile,
}
