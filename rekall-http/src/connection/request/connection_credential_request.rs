use rekall_console::connection::ConnectionCredentialCommand;
use rekall_core::connection::Secret;
use serde::Deserialize;

/// The credential of one connection: a GitLab `token`, a Kubernetes `kubeconfig` with the
/// `context` to keep, a Jira Cloud `email` with its API `token` (a Jira `token` without an email is
/// a Data Center personal access token), or a Grafana `user` with its password in `token` (a Grafana
/// `token` without a user is a service account token). Its `Debug` prints the credentials as `***`.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ConnectionCredentialRequest {
    pub token: Option<Secret>,
    pub kubeconfig: Option<Secret>,
    pub context: Option<String>,
    pub email: Option<String>,
    pub user: Option<String>,
}

impl From<ConnectionCredentialRequest> for ConnectionCredentialCommand {
    fn from(request: ConnectionCredentialRequest) -> Self {
        Self { token: request.token, kubeconfig: request.kubeconfig, context: request.context, email: request.email, user: request.user }
    }
}
