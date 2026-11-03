//! What a person reads about a task's branch while reviewing it, and the one thing they do about
//! its worktree: take it away. Rekall never merges, pushes or deletes the branch.

mod service;
mod view;

pub use service::task_branch_service::TaskBranchService;
pub use view::task_branch_view::TaskBranchView;
