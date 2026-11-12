use rekall_kernel::Id;
use serde::Serialize;

/// Emitted when a task is deleted, with the issue the project's task manager held for it, if any:
/// what the task manager is told so the issue goes too.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDeletedEvent {
    pub project_id: Id,
    pub issue_key: Option<String>,
}

impl crate::StreamEvent for TaskDeletedEvent {
    const FRAME: &'static str = "task-deleted";
}
