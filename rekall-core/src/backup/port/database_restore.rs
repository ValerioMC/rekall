use async_trait::async_trait;
use futures::stream::BoxStream;
use rekall_kernel::Result;

use crate::backup::BackupFile;

/// An uploaded file as it arrives, so a backup of up to a gigabyte never sits in memory whole.
pub type UploadChunks<'a> = BoxStream<'a, Result<Vec<u8>>>;

/// Puts a backup in place of the open database and restarts onto it. Each returns the backup taken
/// of what was there first.
#[async_trait]
pub trait DatabaseRestore: Send + Sync {
    async fn restore(&self, name: &str) -> Result<BackupFile>;

    /// Keeps the upload in the backups folder, then restores it. An empty upload is refused.
    async fn restore_upload(&self, upload: UploadChunks<'_>) -> Result<BackupFile>;
}
