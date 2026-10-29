use rekall_console::filesystem::DirectoryEntryView;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryEntryResponse {
    pub name: String,
    pub path: String,
    pub directory: bool,
    pub hidden: bool,
}

impl From<DirectoryEntryView> for DirectoryEntryResponse {
    fn from(view: DirectoryEntryView) -> Self {
        Self {
            name: view.name,
            path: view.path,
            directory: view.directory,
            hidden: view.hidden,
        }
    }
}
