use rekall_kernel::Id;
use serde::Serialize;

/// Emitted when a project's inventory is rewritten, so an open console reads it again.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryStreamEvent {
    pub project_id: Id,
}

impl crate::StreamEvent for InventoryStreamEvent {
    const FRAME: &'static str = "inventory";
}
