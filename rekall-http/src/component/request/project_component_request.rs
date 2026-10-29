use rekall_console::component::ProjectComponentCommand;
use serde::Deserialize;

/// A component to create or edit. On an edit, a field left out keeps what the component has; on a
/// create, a name left out is the folder's own name.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProjectComponentRequest {
    pub name: Option<String>,
    pub folder: Option<String>,
    pub description: Option<String>,
}

impl From<ProjectComponentRequest> for ProjectComponentCommand {
    fn from(request: ProjectComponentRequest) -> Self {
        Self {
            name: request.name,
            folder: request.folder,
            description: request.description,
        }
    }
}
