use rekall_kernel::Id;
use serde::Serialize;

use crate::context_fill::ContextFillView;

/// A task's whole list of fills after a turn changed one, so the console replaces what it held.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextFillStreamEvent {
    pub task_id: Id,
    pub fills: Vec<ContextFillView>,
}

impl crate::StreamEvent for ContextFillStreamEvent {
    const FRAME: &'static str = "context-fill";
}
