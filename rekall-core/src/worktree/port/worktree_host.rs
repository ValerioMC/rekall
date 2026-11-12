use std::path::Path;

use async_trait::async_trait;
use rekall_kernel::Result;

use crate::worktree::{BranchReview, MergeOutcome, PushOutcome, Worktree, WorktreeStatus};

/// Creates, lists, inspects and removes the worktrees of a repository. It decides nothing: the
/// branch and the folder come from `WorktreeLayout`. An `Err` is an `illegal` the caller can show:
/// git missing, the folder no repository, a name or a folder outside the layout.
#[async_trait]
pub trait WorktreeHost: Send + Sync {
    /// Adds `folder` as a worktree on `branch_name`: a new branch starting at `base`, or the branch
    /// itself when it exists, with the commits it already has. `base` is a branch of the project, or
    /// `HEAD` for the one its folder is on.
    async fn create(&self, project_folder: &Path, branch_name: &str, folder: &Path, base: &str) -> Result<Worktree>;

    /// Takes the worktree's folder away. The branch and its commits stay.
    async fn remove(&self, worktree: &Worktree) -> Result<()>;

    /// The worktrees of the repository that sit under the layout's root.
    async fn list(&self, project_folder: &Path) -> Result<Vec<Worktree>>;

    async fn status(&self, worktree: &Worktree) -> Result<WorktreeStatus>;

    /// What `branch_name` holds beyond `base`, or none when the branch does not exist. Read-only: it
    /// never merges, checks out or deletes anything.
    async fn review_branch(&self, project_folder: &Path, branch_name: &str, worktree_folder: Option<&Path>, base: &str) -> Result<Option<BranchReview>>;

    /// Merges `branch_name` into the project folder's current `HEAD`, which has to be `base` unless
    /// `base` is `HEAD`. A merge git refuses or cannot finish is abandoned before this returns, so
    /// the project is left as it was.
    async fn merge_branch(&self, project_folder: &Path, branch_name: &str, base: &str) -> Result<MergeOutcome>;

    /// Pushes `branch_name` to the project's `origin` and sets it as the branch's upstream. A push
    /// the remote or git turns down is `Refused` with git's first lines, not an error; the message
    /// never carries a credential.
    async fn push_branch(&self, project_folder: &Path, branch_name: &str) -> Result<PushOutcome>;
}
