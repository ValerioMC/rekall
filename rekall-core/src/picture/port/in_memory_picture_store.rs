use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use async_trait::async_trait;
use rekall_kernel::{InternalFailure, Instant, RekallError, Result};

use crate::picture::{PicturePath, PictureStore, StoredPicture};

/// A `PictureStore` that keeps pictures in memory, for tests of what reads and deletes them. It
/// reads no source file, and has a folder only when made with one.
#[derive(Default)]
pub struct InMemoryPictureStore {
    folder: Option<PathBuf>,
    pictures: Mutex<BTreeMap<String, (StoredPicture, Vec<u8>)>>,
    undeletable: Mutex<BTreeSet<String>>,
}

impl InMemoryPictureStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn in_folder() -> Self {
        Self { folder: Some(PathBuf::from("/in-memory/images")), ..Self::default() }
    }

    pub fn put(&self, path: &PicturePath, bytes: &[u8]) {
        self.put_modified_at(path, bytes, Instant::now());
    }

    pub fn put_modified_at(&self, path: &PicturePath, bytes: &[u8], modified_at: Instant) {
        let stored = StoredPicture { path: path.clone(), size_bytes: bytes.len() as u64, modified_at };
        self.pictures.lock().expect("in-memory pictures lock").insert(path.relative().to_string(), (stored, bytes.to_vec()));
    }

    /// Deleting this picture fails from now on.
    pub fn refuse_deleting(&self, path: &PicturePath) {
        self.undeletable.lock().expect("undeletable pictures lock").insert(path.relative().to_string());
    }
}

#[async_trait]
impl PictureStore for InMemoryPictureStore {
    fn folder(&self) -> Option<PathBuf> {
        self.folder.clone()
    }

    async fn read_source(&self, source: &Path, _: u64) -> Result<Vec<u8>> {
        Err(RekallError::illegal(format!("{} cannot be read.", source.display())))
    }

    async fn write(&self, path: &PicturePath, bytes: &[u8]) -> Result<()> {
        self.put(path, bytes);
        Ok(())
    }

    async fn read(&self, path: &PicturePath) -> Result<Vec<u8>> {
        let held = self.pictures.lock().expect("in-memory pictures lock").get(path.relative()).map(|(_, bytes)| bytes.clone());
        held.ok_or_else(|| RekallError::not_found_msg(format!("There is no picture at '{}'.", path.relative())))
    }

    async fn list(&self) -> Result<Vec<StoredPicture>> {
        Ok(self.pictures.lock().expect("in-memory pictures lock").values().map(|(stored, _)| stored.clone()).collect())
    }

    async fn delete(&self, path: &PicturePath) -> Result<()> {
        if self.undeletable.lock().expect("undeletable pictures lock").contains(path.relative()) {
            return Err(RekallError::internal(InternalFailure::Io, format!("{} is held open.", path.relative())));
        }
        self.pictures.lock().expect("in-memory pictures lock").remove(path.relative());
        Ok(())
    }
}
