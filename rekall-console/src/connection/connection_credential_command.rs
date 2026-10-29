use rekall_core::connection::Secret;

/// The credential of one connection: a GitLab `token`, a Kubernetes `kubeconfig` with the
/// `context` to keep, or a Jira Cloud `email` with its API `token` (a Jira `token` without an email is
/// a Data Center personal access token). Its `Debug` prints the credentials as `***`.
#[derive(Debug, Default)]
pub struct ConnectionCredentialCommand {
    pub token: Option<Secret>,
    pub kubeconfig: Option<Secret>,
    pub context: Option<String>,
    pub email: Option<String>,
}
