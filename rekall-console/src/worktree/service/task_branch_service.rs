use rekall_core::worktree::TaskWorktreeService;
use rekall_kernel::{Id, Result};

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

    /// Makes the task's branch and worktree when its project keeps tasks on branches, for a session
    /// opened outside Rekall; none for a project that does not.
    pub async fn start(&self, task_id: Id) -> Result<Option<TaskBranchView>> {
        if !self.worktrees.requires_branch(task_id).await? {
            return Ok(None);
        }
        self.worktrees.ensure(task_id).await?;
        self.view(task_id).await
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/worktree/service/task_branch_service_tests.rs"]
mod tests;
