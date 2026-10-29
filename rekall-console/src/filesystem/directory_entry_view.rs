

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectoryEntryView {
    pub name: String,
    pub path: String,
    pub directory: bool,
    pub hidden: bool,
}
