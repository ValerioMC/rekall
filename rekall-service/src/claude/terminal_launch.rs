use rekall_common::Id;

use super::{TerminalConnection, TerminalTarget};

/// What the PTY needs to start: the folder to start `claude` in and the `/rk` anchors to load.
/// The task fields are empty on a terminal opened on a project alone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerminalLaunch {
    pub project_id: Id,
    pub task_id: Option<Id>,
    pub anchors: String,
    pub working_dir: String,
    pub project_label: String,
    pub task_label: Option<String>,
    pub task_title: Option<String>,
    /// What the project reaches outside Rekall, for the terminal's credential helpers.
    pub connections: Vec<TerminalConnection>,
}

impl TerminalLaunch {
    pub fn target(&self) -> TerminalTarget {
        match self.task_id {
            Some(task_id) => TerminalTarget::Task(task_id),
            None => TerminalTarget::Project(self.project_id),
        }
    }
}
