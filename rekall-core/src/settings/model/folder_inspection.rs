/// What the setup screen needs to know about a folder a person typed, before anything is registered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FolderInspection {
    pub resolved_path: String,
    pub exists: bool,
    pub is_directory: bool,
    pub writable: bool,
    pub has_database: bool,
}

impl FolderInspection {
    pub fn usable(&self) -> bool {
        self.exists && self.is_directory && self.writable
    }
}
