use rekall_console::workingset::OpenTaskView;
use rekall_kernel::Id;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenTaskResponse {
    pub task_id: Id,
    pub project_id: Id,
}

impl From<OpenTaskView> for OpenTaskResponse {
    fn from(view: OpenTaskView) -> Self {
        Self { task_id: view.task_id, project_id: view.project_id }
    }
}
