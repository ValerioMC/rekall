use rekall_kernel::Id;
use serde::Serialize;

use super::{CommitChangesView, WorkingTreeChangesView};

/// What a task changed in one repository: its logged commits, oldest first, and the working tree
/// when it holds changes. No `component_id` is the project's own folder.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryChangesView {
    pub component_id: Option<Id>,
    pub name: String,
    pub commits: Vec<CommitChangesView>,
    pub working_tree: Option<WorkingTreeChangesView>,
}

impl RepositoryChangesView {
    pub fn is_empty(&self) -> bool {
        self.commits.is_empty() && self.working_tree.is_none()
    }
}
