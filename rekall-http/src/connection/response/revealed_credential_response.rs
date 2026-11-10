use rekall_console::connection::RevealedCredentialView;
use serde::Serialize;

/// A stored credential read back on request, the one response in this domain that carries one.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevealedCredentialResponse {
    pub token: Option<String>,
    pub kubeconfig: Option<String>,
    pub email: Option<String>,
    pub user: Option<String>,
}

impl From<RevealedCredentialView> for RevealedCredentialResponse {
    fn from(view: RevealedCredentialView) -> Self {
        Self {
            token: view.token.map(|token| token.expose().to_string()),
            kubeconfig: view.kubeconfig.map(|kubeconfig| kubeconfig.expose().to_string()),
            email: view.email,
            user: view.user,
        }
    }
}
