use rekall_core::dependency::TaskWait;
use rekall_kernel::Id;

/// One task another waits for, and whether it is accepted yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskDependencyView {
    pub task_id: Id,
    pub anchor: String,
    pub title: String,
    pub met: bool,
    pub awaiting_review: bool,
}

impl TaskDependencyView {
    pub fn of(wait: &TaskWait) -> Self {
        Self {
            task_id: wait.task_id,
            anchor: wait.anchor.clone(),
            title: wait.title.clone(),
            met: wait.is_met(),
            awaiting_review: wait.awaits_review(),
        }
    }
}
