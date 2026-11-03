use rekall_core::claude::TerminalView;

use crate::catalog::TaskView;

#[derive(Clone, Debug)]
pub struct QuickTaskView {
    pub task: TaskView,
    pub terminal: TerminalView,
}
