use rekall_console::worktree::TaskBranchView;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskBranchResponse {
    pub branch: String,
    pub commits_ahead: u32,
    pub changed_files: Vec<String>,
    pub merges_cleanly: Option<bool>,
    pub worktree_folder: Option<String>,
    pub has_unsaved_changes: bool,
    pub merge_command: String,
}

impl From<TaskBranchView> for TaskBranchResponse {
    fn from(view: TaskBranchView) -> Self {
        Self {
            branch: view.branch,
            commits_ahead: view.commits_ahead,
            changed_files: view.changed_files,
            merges_cleanly: view.merges_cleanly,
            worktree_folder: view.worktree_folder,
            has_unsaved_changes: view.has_unsaved_changes,
            merge_command: view.merge_command,
        }
    }
}
