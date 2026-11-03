use rekall_core::worktree::BranchReview;

/// A task's branch as the review shows it, with the command that merges it by hand.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskBranchView {
    pub branch: String,
    pub commits_ahead: u32,
    pub changed_files: Vec<String>,
    /// None when git cannot say whether the merge would conflict.
    pub merges_cleanly: Option<bool>,
    pub worktree_folder: Option<String>,
    pub has_unsaved_changes: bool,
    pub merge_command: String,
}

impl TaskBranchView {
    pub fn of(review: BranchReview) -> Self {
        let merge_command = format!("git merge {}", review.branch);
        Self {
            branch: review.branch,
            commits_ahead: review.commits_ahead,
            changed_files: review.changed_files,
            merges_cleanly: review.merges_cleanly,
            worktree_folder: review.worktree_folder,
            has_unsaved_changes: review.has_unsaved_changes,
            merge_command,
        }
    }
}
