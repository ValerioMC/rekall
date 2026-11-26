use serde::Serialize;

use crate::chat::FileChangeKind;

/// One file of a change set. `diff` is a unified diff for an update, and the file's whole text for
/// one added or deleted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileChange {
    pub path: String,
    pub kind: FileChangeKind,
    pub moved_to: Option<String>,
    pub diff: String,
}
