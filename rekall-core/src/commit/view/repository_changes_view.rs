use rekall_kernel::Id;
use serde::Serialize;

use super::{ChangedFileView, CommitChangesView, WorkingTreeChangesView};

/// What a task changed in one repository: its logged commits, oldest first, and the working tree
/// when it holds changes. No `component_id` is the project's own folder.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryChangesView<Files = Vec<ChangedFileView>> {
    pub component_id: Option<Id>,
    pub name: String,
    pub commits: Vec<CommitChangesView<Files>>,
    pub working_tree: Option<WorkingTreeChangesView<Files>>,
}

impl<Files> RepositoryChangesView<Files> {
    pub fn is_empty(&self) -> bool {
        self.commits.is_empty() && self.working_tree.is_none()
    }

    pub(in crate::commit) fn map_files<Mapped: Default>(
        self,
        map: impl Fn(Files) -> Option<Mapped>,
    ) -> RepositoryChangesView<Mapped> {
        RepositoryChangesView {
            component_id: self.component_id,
            name: self.name,
            commits: self.commits.into_iter().map(|commit| commit.map_files(|files| map(files).unwrap_or_default())).collect(),
            working_tree: self.working_tree.and_then(|tree| {
                map(tree.files).map(|files| WorkingTreeChangesView { files, truncated: tree.truncated })
            }),
        }
    }
}
