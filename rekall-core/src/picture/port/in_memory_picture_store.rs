use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use async_trait::async_trait;
use rekall_kernel::{RekallError, Result};

use crate::picture::{PicturePath, PictureStore};

/// A `PictureStore` that keeps pictures in memory, for tests of what reads them. It has no folder
/// and reads no source file.
#[derive(Default)]
pub struct InMemoryPictureStore {
    pictures: Mutex<HashMap<String, Vec<u8>>>,
}

impl InMemoryPictureStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn put(&self, path: &PicturePath, bytes: &[u8]) {
        self.pictures.lock().expect("in-memory pictures lock").insert(path.relative().to_string(), bytes.to_vec());
    }
}

#[async_trait]
impl PictureStore for InMemoryPictureStore {
    fn folder(&self) -> Option<PathBuf> {
        None
    }

    async fn read_source(&self, source: &Path, _: u64) -> Result<Vec<u8>> {
        Err(RekallError::illegal(format!("{} cannot be read.", source.display())))
    }

    async fn write(&self, path: &PicturePath, bytes: &[u8]) -> Result<()> {
        self.put(path, bytes);
        Ok(())
    }

    async fn read(&self, path: &PicturePath) -> Result<Vec<u8>> {
        let held = self.pictures.lock().expect("in-memory pictures lock").get(path.relative()).cloned();
        held.ok_or_else(|| RekallError::not_found_msg(format!("There is no picture at '{}'.", path.relative())))
    }
}
