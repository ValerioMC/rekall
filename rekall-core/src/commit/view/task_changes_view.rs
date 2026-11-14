use serde::Serialize;

use super::{CommitChangesView, WorkingTreeChangesView};

/// Everything a task changed in its project's folder: its logged commits, oldest first, and the
/// working tree when it holds changes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskChangesView {
    pub commits: Vec<CommitChangesView>,
    pub working_tree: Option<WorkingTreeChangesView>,
}
