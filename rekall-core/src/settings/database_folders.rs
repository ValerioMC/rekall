use rekall_kernel::Result;

use super::{DatabaseRegistry, FolderInspection};

/// The registry file and the folders it names.
pub trait DatabaseFolders: Send + Sync {
    /// `None` when nothing has been registered yet.
    fn read(&self) -> Result<Option<DatabaseRegistry>>;

    fn write(&self, registry: &DatabaseRegistry) -> Result<()>;

    /// `~` is the user's home and a relative path is taken from the working directory.
    fn inspect(&self, path: Option<&str>) -> FolderInspection;
}
