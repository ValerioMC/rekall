use rekall_common::{Id, Instant};
use serde::Serialize;

/// A live terminal as the console sees it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalView {
    pub id: Id,
    pub task_id: Id,
    pub step_id: Option<Id>,
    pub anchors: String,
    pub working_dir: String,
    pub project_label: String,
    pub task_label: String,
    pub task_title: String,
    pub skip_permissions: bool,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub live: bool,
    pub started_at: Instant,
    pub last_activity_at: Instant,
}
