use rekall_console::section::ProjectSectionOrderCommand;
use serde::Deserialize;

/// A project's sections, by id, in the order they are to be shown.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProjectSectionOrderRequest {
    pub section_ids: Option<Vec<String>>,
}

impl From<ProjectSectionOrderRequest> for ProjectSectionOrderCommand {
    fn from(request: ProjectSectionOrderRequest) -> Self {
        Self {
            section_ids: request.section_ids,
        }
    }
}
