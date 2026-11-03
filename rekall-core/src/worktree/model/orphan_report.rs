use crate::worktree::Worktree;

/// What reclaiming the worktrees left without a task did: the clean ones removed, the ones with
/// unsaved changes left as they were.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OrphanReport {
    pub removed: Vec<Worktree>,
    pub kept_with_changes: Vec<Worktree>,
}
