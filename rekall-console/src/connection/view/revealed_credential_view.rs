use rekall_core::connection::Secret;

/// A stored credential read back for the person who stored it: a `token` (GitLab, GitHub, Grafana, Jira),
/// a `kubeconfig` (Kubernetes) and, on Jira Cloud, the `email` the token belongs to. Its `Debug` prints `***`.
#[derive(Debug)]
pub struct RevealedCredentialView {
    pub token: Option<Secret>,
    pub kubeconfig: Option<Secret>,
    pub email: Option<String>,
}
