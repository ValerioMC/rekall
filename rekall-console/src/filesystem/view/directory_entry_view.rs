

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectoryEntryView {
    pub name: String,
    pub path: String,
    pub directory: bool,
    pub hidden: bool,
    /// In bytes; none for a folder or a file whose size cannot be read.
    pub size: Option<u64>,
}
