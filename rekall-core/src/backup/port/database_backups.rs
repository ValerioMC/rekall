use async_trait::async_trait;
use rekall_kernel::Result;

use crate::backup::{BackupFile, BackupStatus};

/// The backups of the open database. A database that is not a file on disk has none, and every
/// call but `status` refuses it as a conflict.
#[async_trait]
pub trait DatabaseBackups: Send + Sync {
    fn status(&self) -> Result<BackupStatus>;

    async fn back_up_now(&self) -> Result<BackupFile>;

    /// The zip behind a name the folder handed out; any other name is refused, so a name is never a path.
    async fn read(&self, name: &str) -> Result<Vec<u8>>;
}
