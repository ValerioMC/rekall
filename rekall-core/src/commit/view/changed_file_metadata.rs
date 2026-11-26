use serde::Serialize;

use super::ChangeKind;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangedFileMetadata {
    pub path: String,
    pub old_path: Option<String>,
    pub kind: ChangeKind,
    pub binary: bool,
    pub additions: u32,
    pub deletions: u32,
}
