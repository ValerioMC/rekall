use rekall_core::worktree::{Release, TaskWorktreeService};
use rekall_kernel::{Id, RekallError, Result};

use crate::worktree::TaskBranchView;

#[derive(Clone)]
pub struct TaskBranchService {
    worktrees: TaskWorktreeService,
}

impl TaskBranchService {
    pub fn new(worktrees: TaskWorktreeService) -> Self {
        Self { worktrees }
    }

    /// The task's branch while it exists, whether or not its worktree does; none for a task that
    /// never had one.
    pub async fn view(&self, task_id: Id) -> Result<Option<TaskBranchView>> {
        Ok(self.worktrees.review_branch(task_id).await?.map(TaskBranchView::of))
    }

    /// Takes the worktree away and keeps the branch. Unsaved changes in it refuse the removal.
    pub async fn remove_worktree(&self, task_id: Id) -> Result<Option<TaskBranchView>> {
        match self.worktrees.release(task_id).await? {
            Release::KeptWithChanges => {
                Err(RekallError::conflict("The worktree has unsaved changes. Save or discard them, then remove it."))
            }
            Release::Removed | Release::NoWorktree => self.view(task_id).await,
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/worktree/service/task_branch_service_tests.rs"]
mod tests;
