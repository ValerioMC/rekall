use rekall_common::Id;
use serde::Serialize;

use super::TaskStepView;

/// A task's whole checklist after a write, from the console or from MCP alike.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepStreamEvent {
    pub task_id: Id,
    pub steps: Vec<TaskStepView>,
}
