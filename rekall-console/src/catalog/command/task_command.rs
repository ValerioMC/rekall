use rekall_kernel::Id;
use rekall_store::{TaskKind, TaskStatus};

#[derive(Clone, Debug, Default)]
pub struct TaskCommand {
    pub label: Option<String>,
    pub title: Option<String>,
    pub status: Option<TaskStatus>,
    pub description: Option<String>,
    pub project_id: Option<Id>,
    pub tag_id: Option<Id>,
    pub kind: Option<TaskKind>,
    pub epic_id: Option<Id>,
}
