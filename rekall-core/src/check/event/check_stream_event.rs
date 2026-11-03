use rekall_kernel::Id;
use serde::Serialize;

/// Emitted when a project's checks change, so an open console reads them again. It names the
/// project, not the check: the list is small and ordered.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckStreamEvent {
    pub project_id: Id,
}

impl crate::StreamEvent for CheckStreamEvent {
    const FRAME: &'static str = "project-check";
}
