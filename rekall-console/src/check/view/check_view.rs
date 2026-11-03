use rekall_kernel::{Id, Instant};
use rekall_store::done_check;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckView {
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

impl CheckView {
    pub fn of(row: &done_check::Model) -> Self {
        Self {
            id: row.id,
            project_id: row.project_id,
            step_id: row.step_id,
            label: row.label.clone(),
            command: row.command.clone(),
            folder: row.folder.clone(),
            timeout_seconds: row.timeout_seconds,
            position: row.position,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
