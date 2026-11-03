use rekall_console::dependency::TaskDependencyView;
use rekall_kernel::Id;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDependencyResponse {
    pub task_id: Id,
    pub anchor: String,
    pub title: String,
    pub met: bool,
    pub awaiting_review: bool,
}

impl From<TaskDependencyView> for TaskDependencyResponse {
    fn from(view: TaskDependencyView) -> Self {
        Self { task_id: view.task_id, anchor: view.anchor, title: view.title, met: view.met, awaiting_review: view.awaiting_review }
    }
}
