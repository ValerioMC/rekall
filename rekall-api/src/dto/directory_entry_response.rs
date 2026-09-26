use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryEntryResponse {
    pub name: String,
    pub path: String,
    pub directory: bool,
    pub hidden: bool,
}
