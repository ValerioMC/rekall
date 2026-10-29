use rekall_common::Id;
use rekall_console::catalog::TaskCommand;
use rekall_store::{TaskKind, TaskStatus};
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TaskRequest {
    pub label: Option<String>,
    pub title: Option<String>,
    pub status: Option<TaskStatus>,
    pub description: Option<String>,
    pub project_id: Option<Id>,
    pub tag_id: Option<Id>,
    pub kind: Option<TaskKind>,
    pub epic_id: Option<Id>,
}

impl From<TaskRequest> for TaskCommand {
    fn from(request: TaskRequest) -> Self {
        Self {
            label: request.label,
            title: request.title,
            status: request.status,
            description: request.description,
            project_id: request.project_id,
            tag_id: request.tag_id,
            kind: request.kind,
            epic_id: request.epic_id,
        }
    }
}
