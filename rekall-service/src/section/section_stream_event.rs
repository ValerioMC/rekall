use rekall_common::Id;
use serde::Serialize;

/// Emitted when a project's sections change, from a session or the console, so an open console
/// reads them again. It names the project, not the section: the list is small and ordered.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionStreamEvent {
    pub project_id: Id,
}

impl crate::StreamEvent for SectionStreamEvent {
    const FRAME: &'static str = "project-section";
}
