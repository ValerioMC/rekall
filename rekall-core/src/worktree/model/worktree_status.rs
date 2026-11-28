/// What a worktree holds that the project's own branch does not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorktreeStatus {
    /// Every modified, new or deleted file git does not ignore, by its path in the repository.
    pub unsaved_files: Vec<String>,
    /// Commits on the worktree's branch that the project folder's current `HEAD` does not have.
    pub commits_ahead: u32,
}

impl WorktreeStatus {
    pub fn has_unsaved_changes(&self) -> bool {
        !self.unsaved_files.is_empty()
    }
}
