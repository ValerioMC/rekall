use rekall_console::section::ProjectSectionCommand;
use serde::Deserialize;

/// A section to create or edit. On an edit, a field left out keeps what the section has; on a
/// create, a context left out puts the section in the project's first one.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProjectSectionRequest {
    pub title: Option<String>,
    pub body_markdown: Option<String>,
    pub context_id: Option<String>,
}

impl From<ProjectSectionRequest> for ProjectSectionCommand {
    fn from(request: ProjectSectionRequest) -> Self {
        Self {
            title: request.title,
            body_markdown: request.body_markdown,
            context_id: request.context_id,
        }
    }
}
