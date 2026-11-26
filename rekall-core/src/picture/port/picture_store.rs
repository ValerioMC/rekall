use std::path::{Path, PathBuf};

use async_trait::async_trait;
use rekall_kernel::Result;

use crate::picture::PicturePath;

/// The pictures on disk beside the database. A database that is not a file has no place for them,
/// and every call refuses it as a conflict.
#[async_trait]
pub trait PictureStore: Send + Sync {
    /// The folder the pictures are in, as a real path; none when the database is not a file.
    fn folder(&self) -> Option<PathBuf>;

    /// A file the person chose anywhere on this machine. A relative path, a missing one, a folder
    /// and a file past `limit_bytes` are refused.
    async fn read_source(&self, source: &Path, limit_bytes: u64) -> Result<Vec<u8>>;

    /// Writes even a file that is already there, which renews its modification time.
    async fn write(&self, path: &PicturePath, bytes: &[u8]) -> Result<()>;

    async fn read(&self, path: &PicturePath) -> Result<Vec<u8>>;
}
