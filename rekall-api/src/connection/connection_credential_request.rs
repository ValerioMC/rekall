use rekall_service::connection::Secret;
use serde::Deserialize;

/// The credential of one connection: a GitLab `token`, or a Kubernetes `kubeconfig` with the
/// `context` to keep. Its `Debug` prints the credentials as `***`.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ConnectionCredentialRequest {
    pub token: Option<Secret>,
    pub kubeconfig: Option<Secret>,
    pub context: Option<String>,
}
