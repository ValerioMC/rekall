use rekall_common::{Id, Instant};
use rekall_console::connection::ConnectionAccessView;
use serde::Serialize;

/// One request a terminal made for the credential: when, from which terminal and task, and whether it was handed out.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionAccessResponse {
    pub id: Id,
    pub accessed_at: Instant,
    pub granted: bool,
    pub reason: Option<String>,
    pub terminal_id: String,
    pub task_id: Option<Id>,
    pub task_label: Option<String>,
    pub task_title: Option<String>,
}

impl From<ConnectionAccessView> for ConnectionAccessResponse {
    fn from(view: ConnectionAccessView) -> Self {
        Self {
            id: view.id,
            accessed_at: view.accessed_at,
            granted: view.granted,
            reason: view.reason,
            terminal_id: view.terminal_id,
            task_id: view.task_id,
            task_label: view.task_label,
            task_title: view.task_title,
        }
    }
}
