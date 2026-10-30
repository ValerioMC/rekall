use rekall_kernel::{Id, Instant};
use serde::Serialize;

use crate::claude::{SessionKind, TerminalConnectionView, TerminalTarget};

/// A live session as the console sees it. The task fields are empty on one opened on a project.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalView {
    pub id: Id,
    pub project_id: Id,
    pub task_id: Option<Id>,
    pub step_id: Option<Id>,
    pub anchors: String,
    pub working_dir: String,
    pub project_label: String,
    pub task_label: Option<String>,
    pub task_title: Option<String>,
    pub skip_permissions: bool,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub kind: SessionKind,
    pub live: bool,
    pub started_at: Instant,
    pub last_activity_at: Instant,
    /// What the terminal's project reaches outside Rekall.
    pub connections: Vec<TerminalConnectionView>,
}

impl TerminalView {
    pub fn target(&self) -> TerminalTarget {
        match self.task_id {
            Some(task_id) => TerminalTarget::Task(task_id),
            None => TerminalTarget::Project(self.project_id),
        }
    }
}
