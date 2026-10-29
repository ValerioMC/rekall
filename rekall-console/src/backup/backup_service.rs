use std::sync::Arc;

use rekall_core::backup::{BackupFile, BackupStatus, DatabaseBackups, DatabaseRestore, UploadChunks};
use rekall_kernel::Result;

#[derive(Clone)]
pub struct BackupService {
    backups: Arc<dyn DatabaseBackups>,
    restore: Arc<dyn DatabaseRestore>,
}

impl BackupService {
    pub fn new(backups: Arc<dyn DatabaseBackups>, restore: Arc<dyn DatabaseRestore>) -> Self {
        Self { backups, restore }
    }

    pub fn status(&self) -> Result<BackupStatus> {
        self.backups.status()
    }

    pub async fn back_up_now(&self) -> Result<BackupFile> {
        self.backups.back_up_now().await
    }

    pub async fn download(&self, name: &str) -> Result<Vec<u8>> {
        self.backups.read(name).await
    }

    /// Returns the backup taken of what was there; the application restarts onto the restored file.
    pub async fn restore(&self, name: &str) -> Result<BackupFile> {
        self.restore.restore(name).await
    }

    pub async fn restore_upload(&self, upload: UploadChunks<'_>) -> Result<BackupFile> {
        self.restore.restore_upload(upload).await
    }
}

#[cfg(test)]
#[path = "../../tests/unit/backup/backup_service_tests.rs"]
mod tests;
