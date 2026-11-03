/// What a task's branch holds that the project's current `HEAD` does not, read without changing
/// anything.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BranchReview {
    pub branch: String,
    pub commits_ahead: u32,
    /// The files the branch changes since it left the project's `HEAD`.
    pub changed_files: Vec<String>,
    /// Whether merging the branch into `HEAD` would conflict; none when git cannot say.
    pub merges_cleanly: Option<bool>,
    /// The worktree's folder while it exists.
    pub worktree_folder: Option<String>,
    pub has_unsaved_changes: bool,
}
