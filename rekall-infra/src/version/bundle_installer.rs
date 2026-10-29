use std::path::Path;

use async_trait::async_trait;

/// A step of the install that failed, with what the system said.
#[derive(Debug, thiserror::Error)]
#[error("{action}: {cause}")]
pub struct BundleError {
    pub action: String,
    pub cause: String,
}

impl BundleError {
    pub fn new(action: &str, cause: impl ToString) -> Self {
        Self { action: action.to_string(), cause: cause.to_string() }
    }
}

/// The system tools an update needs to unpack a disk image and copy the app out of it.
#[async_trait]
pub trait BundleInstaller: Send + Sync {
    /// Mounts `image` read-only at `mount_point`, without opening a Finder window.
    async fn attach(&self, image: &Path, mount_point: &Path) -> Result<(), BundleError>;

    async fn detach(&self, mount_point: &Path) -> Result<(), BundleError>;

    /// Copies an `.app` bundle with its signature, permissions and symlinks intact.
    async fn copy_bundle(&self, source: &Path, target: &Path) -> Result<(), BundleError>;
}
