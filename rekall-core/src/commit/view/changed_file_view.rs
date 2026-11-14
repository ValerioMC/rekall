use serde::Serialize;

use super::{ChangeKind, DiffHunkView};

/// One file of a patch. A binary file has no hunks, and `old_path` is set only for a rename.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangedFileView {
    pub path: String,
    pub old_path: Option<String>,
    pub kind: ChangeKind,
    pub binary: bool,
    pub additions: u32,
    pub deletions: u32,
    pub hunks: Vec<DiffHunkView>,
}
