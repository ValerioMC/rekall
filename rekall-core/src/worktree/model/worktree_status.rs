/// What a worktree holds that the project's own branch does not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorktreeStatus {
    pub has_unsaved_changes: bool,
    /// Commits on the worktree's branch that the project folder's current `HEAD` does not have.
    pub commits_ahead: u32,
}
