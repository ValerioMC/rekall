use rekall_common::Id;
use rekall_model::{TaskKind, TaskStatus};

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
