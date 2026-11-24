use rekall_console::workingset::PlaceView;
use rekall_kernel::Id;
use serde::Serialize;

/// A place in a project: one of its tasks, or its task list when `taskId` is null.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceResponse {
    pub project_id: Id,
    pub task_id: Option<Id>,
}

impl From<PlaceView> for PlaceResponse {
    fn from(view: PlaceView) -> Self {
        Self { project_id: view.project_id, task_id: view.task_id }
    }
}
