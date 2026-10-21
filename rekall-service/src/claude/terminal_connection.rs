use rekall_common::Id;
use rekall_model::{external_connection, project_connection, ConnectionKind};

/// A connection the terminal's project binds, as the terminal's credential helpers need it: which
/// keychain account to ask, and where a GitLab push goes. Never the credential itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerminalConnection {
    pub connection_id: Id,
    pub kind: ConnectionKind,
    pub label: String,
    /// The GitLab project path or the Kubernetes namespace.
    pub target: Option<String>,
    /// GitLab only: the instance pushes and API calls go to.
    pub instance_url: Option<String>,
    pub secret_ref: String,
    pub credential_set: bool,
}

impl TerminalConnection {
    pub fn of(binding: &project_connection::Model, connection: &external_connection::Model) -> Self {
        Self {
            connection_id: connection.id,
            kind: connection.kind,
            label: connection.label.clone(),
            target: binding.target.clone(),
            instance_url: (connection.kind == ConnectionKind::Gitlab).then(|| connection.base_url.clone()).flatten(),
            secret_ref: connection.secret_ref.clone(),
            credential_set: connection.has_credential(),
        }
    }
}
