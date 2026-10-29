use std::path::{Path, PathBuf};

use crate::bootstrap::folder::absolute_normalised;

/// Where the open database lives on disk: the folder that holds it and its file's name. Only a
/// file database has one; an in-memory one, which the setup screen runs on, has none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatabaseLocation {
    pub folder: PathBuf,
    pub data_file_name: String,
}

impl DatabaseLocation {
    pub fn of(data_file: &Path) -> Option<Self> {
        let absolute = absolute_normalised(data_file);
        let folder = absolute.parent()?.to_path_buf();
        let data_file_name = absolute.file_name()?.to_string_lossy().into_owned();
        Some(Self { folder, data_file_name })
    }

    pub fn backups(&self) -> PathBuf {
        self.folder.join("backups")
    }

    pub fn data_file(&self) -> PathBuf {
        self.folder.join(&self.data_file_name)
    }
}
