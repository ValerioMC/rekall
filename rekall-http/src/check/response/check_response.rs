use rekall_console::check::CheckView;
use rekall_kernel::{Id, Instant};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckResponse {
    pub id: Id,
    pub project_id: Id,
    pub step_id: Option<Id>,
    pub label: String,
    pub command: String,
    pub folder: Option<String>,
    pub timeout_seconds: i32,
    pub position: i32,
    pub created_at: Instant,
    pub updated_at: Instant,
}

impl From<CheckView> for CheckResponse {
    fn from(view: CheckView) -> Self {
        Self {
            id: view.id,
            project_id: view.project_id,
            step_id: view.step_id,
            label: view.label,
            command: view.command,
            folder: view.folder,
            timeout_seconds: view.timeout_seconds,
            position: view.position,
            created_at: view.created_at,
            updated_at: view.updated_at,
        }
    }
}
