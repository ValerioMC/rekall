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
}

#[cfg(test)]
#[path = "../../../tests/unit/worktree/service/task_branch_service_tests.rs"]
mod tests;
