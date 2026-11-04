use rekall_console::deviation::TaskDeviationView;
use rekall_kernel::{Id, Instant};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDeviationResponse {
    pub id: Id,
    pub task_id: Id,
    pub step_number: i32,
    pub step_title: Option<String>,
    pub body_markdown: String,
    pub created_at: Instant,
    pub updated_at: Instant,
}

impl From<TaskDeviationView> for TaskDeviationResponse {
    fn from(view: TaskDeviationView) -> Self {
        Self {
            id: view.id,
            task_id: view.task_id,
            step_number: view.step_number,
            step_title: view.step_title,
            body_markdown: view.body_markdown,
            created_at: view.created_at,
            updated_at: view.updated_at,
        }
    }
}
