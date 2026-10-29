//! File operations on `.app` bundles and the update's scratch folder. A bundle is a directory, so
//! removing one removes its whole tree.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use super::BundleError;

/// The first `.app` directory directly inside `folder`.
pub async fn find_app_in(folder: &Path) -> Result<PathBuf, BundleError> {
    let unreadable = |failure: std::io::Error| BundleError::new("the disk image could not be read", failure);
    let mut entries = tokio::fs::read_dir(folder).await.map_err(unreadable)?;
    while let Some(entry) = entries.next_entry().await.map_err(unreadable)? {
        let path = entry.path();
        if path.extension().is_some_and(|extension| extension == "app") && path.is_dir() {
            return Ok(path);
        }
    }
    Err(BundleError::new("the disk image could not be read", "it holds no Rekall.app"))
}

/// Leaves `folder` existing and empty.
pub async fn reset_folder(folder: &Path) -> Result<(), BundleError> {
    remove_tree(folder).await?;
    tokio::fs::create_dir_all(folder)
        .await
        .map_err(|failure| BundleError::new(&format!("{} could not be created", folder.display()), failure))
}

/// Removes a file or a directory tree; one already gone is not an error.
pub async fn remove_tree(path: &Path) -> Result<(), BundleError> {
    let outcome = match tokio::fs::symlink_metadata(path).await {
        Err(failure) if failure.kind() == ErrorKind::NotFound => return Ok(()),
        Err(failure) => Err(failure),
        Ok(metadata) if metadata.is_dir() => tokio::fs::remove_dir_all(path).await,
        Ok(_) => tokio::fs::remove_file(path).await,
    };
    outcome.map_err(|failure| BundleError::new(&format!("{} could not be deleted", path.display()), failure))
}

pub async fn rename(from: &Path, to: &Path) -> Result<(), BundleError> {
    tokio::fs::rename(from, to)
        .await
        .map_err(|failure| BundleError::new(&format!("{} could not be moved to {}", from.display(), to.display()), failure))
}
