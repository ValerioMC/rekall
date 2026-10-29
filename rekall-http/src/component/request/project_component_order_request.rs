use rekall_console::component::ProjectComponentOrderCommand;
use serde::Deserialize;

/// A project's components, by id, in the order they are to be shown.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProjectComponentOrderRequest {
    pub component_ids: Option<Vec<String>>,
}

impl From<ProjectComponentOrderRequest> for ProjectComponentOrderCommand {
    fn from(request: ProjectComponentOrderRequest) -> Self {
        Self {
            component_ids: request.component_ids,
        }
    }
}
