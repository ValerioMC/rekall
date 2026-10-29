use rekall_kernel::Id;
use rekall_store::{external_connection, project_connection, ConnectionKind};

/// A connection the terminal's project binds, as the terminal's credential helpers need it: which
/// keychain account to ask, and where a GitLab push goes. Never the credential itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerminalConnection {
    pub connection_id: Id,
    pub kind: ConnectionKind,
    pub label: String,
    /// The GitLab project path, the Kubernetes namespace or the Jira project key.
    pub target: Option<String>,
    /// GitLab and Jira only: the instance or site pushes and API calls go to.
    pub instance_url: Option<String>,
    /// Jira only: the proxy API calls go through.
    pub proxy_url: Option<String>,
    pub secret_ref: String,
    pub credential_set: bool,
}

impl TerminalConnection {
    pub fn of(binding: &project_connection::Model, connection: &external_connection::Model) -> Self {
        Self {
            connection_id: connection.id,
            kind: connection.kind,
            label: binding.label.clone(),
            target: binding.target.clone(),
            instance_url: connection.kind.reached_at_typed_url().then(|| connection.base_url.clone()).flatten(),
            proxy_url: connection.proxy_url.clone(),
            secret_ref: connection.secret_ref.clone(),
            credential_set: connection.has_credential(),
        }
    }
}
