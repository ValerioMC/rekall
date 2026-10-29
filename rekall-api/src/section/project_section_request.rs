use rekall_console::section::ProjectSectionCommand;
use serde::Deserialize;

/// A section to create or edit. On an edit, a field left out keeps what the section has.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProjectSectionRequest {
    pub title: Option<String>,
    pub body_markdown: Option<String>,
}

impl From<ProjectSectionRequest> for ProjectSectionCommand {
    fn from(request: ProjectSectionRequest) -> Self {
        Self {
            title: request.title,
            body_markdown: request.body_markdown,
        }
    }
}
