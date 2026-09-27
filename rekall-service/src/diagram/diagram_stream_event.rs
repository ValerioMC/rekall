use rekall_common::Id;
use serde::Serialize;

use super::DiagramSummaryView;

/// Emitted when a diagram is written or deleted, so an open console lists it without a reload.
/// On a delete `diagram` is null.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagramStreamEvent {
    pub diagram_id: Id,
    pub diagram: Option<DiagramSummaryView>,
    pub deleted: bool,
}
