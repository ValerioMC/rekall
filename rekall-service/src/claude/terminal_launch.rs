use rekall_common::Id;

/// What the PTY needs to start: the folder to start `claude` in and the `/rk` anchors to load.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerminalLaunch {
    pub task_id: Id,
    pub anchors: String,
    pub working_dir: String,
    pub project_label: String,
    pub task_label: String,
    pub task_title: String,
}
