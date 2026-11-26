use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rekall_core::picture::{PicturePath, PictureStore, StoredPicture};
use rekall_kernel::{InternalFailure, Instant, RekallError, Result};
use tokio::io::AsyncReadExt;
use tracing::warn;

use crate::backup::DatabaseLocation;
use crate::picture::picture_files;

const BYTES_PER_MEGABYTE: u64 = 1024 * 1024;

/// Pictures in the `images` folder beside the open database.
#[derive(Clone)]
pub struct FolderPictureStore {
    root: Option<PathBuf>,
}

impl FolderPictureStore {
    pub fn new(location: Option<&DatabaseLocation>) -> Self {
        Self { root: location.map(DatabaseLocation::images) }
    }

    fn root(&self) -> Result<&Path> {
        self.root
            .as_deref()
            .ok_or_else(|| RekallError::conflict("The open database is not a file on disk, so it keeps no pictures."))
    }

    fn file_of(&self, path: &PicturePath) -> Result<PathBuf> {
        Ok(self.root()?.join(path.relative()))
    }
}

#[async_trait]
impl PictureStore for FolderPictureStore {
    /// Resolved, since Claude Code matches a read rule against the real path, even before the
    /// first picture creates the folder.
    fn folder(&self) -> Option<PathBuf> {
        self.root.as_deref().map(real_path)
    }

    async fn read_source(&self, source: &Path, limit_bytes: u64) -> Result<Vec<u8>> {
        self.root()?;
        if !source.is_absolute() {
            return Err(RekallError::illegal(format!("{} is not an absolute path.", source.display())));
        }
        let found = tokio::fs::metadata(source).await.map_err(|failure| unreadable(source, failure))?;
        if !found.is_file() {
            return Err(RekallError::illegal(format!("{} is not a file.", source.display())));
        }
        let bytes = read_up_to(source, limit_bytes + 1).await.map_err(|failure| unreadable(source, failure))?;
        if bytes.len() as u64 > limit_bytes {
            return Err(RekallError::illegal(format!("{} is larger than {} MB.", source.display(), limit_bytes / BYTES_PER_MEGABYTE)));
        }
        Ok(bytes)
    }

    async fn write(&self, path: &PicturePath, bytes: &[u8]) -> Result<()> {
        let file = self.file_of(path)?;
        let folder = file.parent().expect("a picture path has folders above its file");
        tokio::fs::create_dir_all(folder).await.map_err(disk_failure)?;
        tokio::fs::write(&file, bytes).await.map_err(disk_failure)
    }

    async fn read(&self, path: &PicturePath) -> Result<Vec<u8>> {
        tokio::fs::read(self.file_of(path)?).await.map_err(|failure| match failure.kind() {
            ErrorKind::NotFound | ErrorKind::NotADirectory | ErrorKind::IsADirectory => {
                RekallError::not_found_msg(format!("There is no picture at '{}'.", path.relative()))
            }
            _ => disk_failure(failure),
        })
    }

    async fn list(&self) -> Result<Vec<StoredPicture>> {
        let images = self.root()?.to_path_buf();
        tokio::task::spawn_blocking(move || stored_pictures(&images))
            .await
            .map_err(|failure| RekallError::internal(InternalFailure::Background, failure))?
            .map_err(disk_failure)
    }

    async fn delete(&self, path: &PicturePath) -> Result<()> {
        let file = self.file_of(path)?;
        match tokio::fs::remove_file(&file).await {
            Err(failure) if failure.kind() != ErrorKind::NotFound => Err(disk_failure(failure)),
            _ => {
                remove_emptied_folders(self.root()?, &file).await;
                Ok(())
            }
        }
    }
}

/// A file removed while the folder is being listed is left out.
fn stored_pictures(images: &Path) -> io::Result<Vec<StoredPicture>> {
    let mut stored = Vec::new();
    for path in picture_files(images)? {
        match std::fs::symlink_metadata(images.join(path.relative())) {
            Ok(found) => {
                let modified_at = Instant::from_datetime(DateTime::<Utc>::from(found.modified()?));
                stored.push(StoredPicture { path, size_bytes: found.len(), modified_at });
            }
            Err(failure) if failure.kind() == ErrorKind::NotFound => {}
            Err(failure) => return Err(failure),
        }
    }
    Ok(stored)
}

/// From the file's folder upwards, until one still holds something. The picture is gone by then,
/// so a folder that cannot be removed is only logged.
async fn remove_emptied_folders(images: &Path, file: &Path) {
    for folder in file.ancestors().skip(1).take_while(|folder| *folder != images) {
        if let Err(failure) = tokio::fs::remove_dir(folder).await {
            if failure.kind() != ErrorKind::DirectoryNotEmpty {
                warn!(%failure, folder = %folder.display(), "Could not remove an emptied picture folder");
            }
            return;
        }
    }
}

fn real_path(folder: &Path) -> PathBuf {
    let under_real_parent = || Some(folder.parent()?.canonicalize().ok()?.join(folder.file_name()?));
    folder.canonicalize().ok().or_else(under_real_parent).unwrap_or_else(|| folder.to_path_buf())
}

async fn read_up_to(source: &Path, bytes: u64) -> io::Result<Vec<u8>> {
    let mut read = Vec::new();
    tokio::fs::File::open(source).await?.take(bytes).read_to_end(&mut read).await?;
    Ok(read)
}

fn unreadable(source: &Path, failure: io::Error) -> RekallError {
    RekallError::illegal(format!("{} cannot be read: {failure}", source.display()))
}

fn disk_failure(failure: io::Error) -> RekallError {
    RekallError::internal(InternalFailure::Io, failure)
}

#[cfg(test)]
#[path = "../../tests/unit/picture/folder_picture_store_tests.rs"]
mod tests;
