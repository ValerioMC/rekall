//! Git worktrees for `rekall-core`'s `WorktreeHost` port.

mod git_worktree_host;
mod worktree_cleaner;

pub use git_worktree_host::GitWorktreeHost;
pub use worktree_cleaner::WorktreeCleaner;
