use rekall_service::connection::Secret;
use serde::Deserialize;

/// The credential of one connection: a GitLab `token`, a Kubernetes `kubeconfig` with the
/// `context` to keep, or a Jira `email` with its API `token`. Its `Debug` prints the credentials as `***`.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ConnectionCredentialRequest {
    pub token: Option<Secret>,
    pub kubeconfig: Option<Secret>,
    pub context: Option<String>,
    pub email: Option<String>,
}
