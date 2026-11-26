use std::path::PathBuf;

/// Where the pictures a text shows are on disk, as a session is told. There is no folder when the
/// database is not a file.
#[derive(Clone, Debug, Default)]
pub struct SessionPictures {
    pub(in crate::context) folder: Option<PathBuf>,
}

impl SessionPictures {
    pub fn new(folder: Option<PathBuf>) -> Self {
        Self { folder }
    }
}
