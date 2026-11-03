use std::path::Path;

use async_trait::async_trait;
use rekall_kernel::Result;

use crate::worktree::{Worktree, WorktreeStatus};

/// Creates, lists, inspects and removes the worktrees of a repository. It decides nothing: the
/// branch and the folder come from `WorktreeLayout`. An `Err` is an `illegal` the caller can show:
/// git missing, the folder no repository, a name or a folder outside the layout.
#[async_trait]
pub trait WorktreeHost: Send + Sync {
    /// Adds `folder` as a worktree on `branch_name`: a new branch starting at the project's `HEAD`, or the
    /// branch itself when it exists, with the commits it already has.
    async fn create(&self, project_folder: &Path, branch_name: &str, folder: &Path) -> Result<Worktree>;

    /// Takes the worktree's folder away. The branch and its commits stay.
    async fn remove(&self, worktree: &Worktree) -> Result<()>;

    /// The worktrees of the repository that sit under the layout's root.
    async fn list(&self, project_folder: &Path) -> Result<Vec<Worktree>>;

    async fn status(&self, worktree: &Worktree) -> Result<WorktreeStatus>;
}
