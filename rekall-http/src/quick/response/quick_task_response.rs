use rekall_console::quick::QuickTaskView;
use rekall_core::claude::TerminalView;
use serde::Serialize;

use crate::catalog::TaskResponse;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickTaskResponse {
    pub task: TaskResponse,
    pub terminal: TerminalView,
}

impl From<QuickTaskView> for QuickTaskResponse {
    fn from(view: QuickTaskView) -> Self {
        Self { task: view.task.into(), terminal: view.terminal }
    }
}
