use rekall_model::{external_connection, project_connection, ConnectionKind};

/// A connection as a session's context names it. It has no field that could hold a credential,
/// the keychain account, a cluster's server or its CA: a Kubernetes connection is reached by label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectionView {
    pub kind: ConnectionKind,
    pub label: String,
    pub title: String,
    /// The GitLab project path or the Kubernetes namespace the project uses it for.
    pub target: Option<String>,
    /// GitLab only: the instance a push or an API call goes to.
    pub instance_url: Option<String>,
    pub credential_set: bool,
}

impl ConnectionView {
    pub fn of(binding: &project_connection::Model, connection: &external_connection::Model) -> Self {
        Self {
            kind: connection.kind,
            label: connection.label.clone(),
            title: connection.title.clone(),
            target: binding.target.clone(),
            instance_url: (connection.kind == ConnectionKind::Gitlab).then(|| connection.base_url.clone()).flatten(),
            credential_set: connection.has_credential(),
        }
    }
}
