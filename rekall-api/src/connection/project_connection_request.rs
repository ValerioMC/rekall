use rekall_kernel::{Id, RekallError};
use rekall_console::connection::ProjectConnectionCommand;
use rekall_web::error::{ApiError, ApiResult};
use rekall_web::extract::uuid;
use serde::Deserialize;

/// A binding as the console sends it: the connection (only when binding), the label a session
/// names it by, and what the project uses it for: the GitLab project path, the Kubernetes
/// namespace or the Jira project key.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProjectConnectionRequest {
    pub connection_id: Option<String>,
    pub label: Option<String>,
    pub target: Option<String>,
}

impl ProjectConnectionRequest {
    pub fn connection_id(&self) -> ApiResult<Id> {
        uuid("connectionId", self.connection_id.as_deref().unwrap_or(""))
            .map_err(|_| ApiError::Domain(RekallError::illegal("A binding names the connection it uses as connectionId.")))
    }
}

impl From<ProjectConnectionRequest> for ProjectConnectionCommand {
    fn from(request: ProjectConnectionRequest) -> Self {
        Self { label: request.label, target: request.target }
    }
}
