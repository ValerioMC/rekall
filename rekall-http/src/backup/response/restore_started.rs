use rekall_core::backup::BackupFile;
use serde::Serialize;

/// A restore that was accepted: the application is restarting, and this backup holds what was there.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::backup) struct RestoreStarted {
    pub(in crate::backup) restarting: bool,
    pub(in crate::backup) previous_state: BackupFile,
}
