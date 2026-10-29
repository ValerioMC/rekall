use rekall_common::{Id, Instant};
use rekall_model::{connection_access, task};

/// One request a terminal made for the credential: when, from which terminal and task, and whether it was handed out.
#[derive(Clone, Debug)]
pub struct ConnectionAccessView {
    pub id: Id,
    pub accessed_at: Instant,
    pub granted: bool,
    pub reason: Option<String>,
    pub terminal_id: String,
    pub task_id: Option<Id>,
    pub task_label: Option<String>,
    pub task_title: Option<String>,
}

impl ConnectionAccessView {
    pub fn of(access: &connection_access::Model, task: Option<&task::Model>) -> Self {
        Self {
            id: access.id,
            accessed_at: access.accessed_at,
            granted: access.granted,
            reason: access.reason.clone(),
            terminal_id: access.terminal_id.clone(),
            task_id: access.task_id,
            task_label: task.map(|task| task.label.clone()),
            task_title: task.map(|task| task.title.clone()),
        }
    }
}
