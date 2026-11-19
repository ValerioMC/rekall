use serde::Serialize;

use super::ChangedFileView;

/// What the project's folder holds that no commit does yet, untracked files included.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkingTreeChangesView {
    pub truncated: bool,
    pub files: Vec<ChangedFileView>,
}
