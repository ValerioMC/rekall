use serde::Deserialize;

/// What a project uses a connection for: the GitLab project path or the Kubernetes namespace.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProjectConnectionRequest {
    pub target: Option<String>,
}
