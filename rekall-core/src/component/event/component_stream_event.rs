use rekall_kernel::Id;
use serde::Serialize;

/// Emitted when a project's components change, from a session or the console, so an open console
/// reads them again. It names the project, not the component: the list is small and ordered.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentStreamEvent {
    pub project_id: Id,
}

impl crate::StreamEvent for ComponentStreamEvent {
    const FRAME: &'static str = "project-component";
}
