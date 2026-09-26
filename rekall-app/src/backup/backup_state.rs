use super::{DatabaseBackupService, DatabaseRestoreService};

#[derive(Clone)]
pub struct BackupState {
    pub backups: DatabaseBackupService,
    pub restorer: DatabaseRestoreService,
}
