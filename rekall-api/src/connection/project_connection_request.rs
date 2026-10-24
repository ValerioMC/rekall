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
