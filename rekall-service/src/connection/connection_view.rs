use rekall_model::{external_connection, project_connection, ConnectionKind};

/// A connection as a session's context names it. It has no field that could hold a credential,
/// the keychain account, a cluster's server or its CA: a Kubernetes connection is reached by label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectionView {
    pub kind: ConnectionKind,
    pub label: String,
    pub title: String,
    /// The GitLab project path, the Kubernetes namespace or the Jira project key the project uses it for.
    pub target: Option<String>,
    /// GitLab and Jira only: the instance or site an API call goes to.
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
            instance_url: connection.kind.reached_at_typed_url().then(|| connection.base_url.clone()).flatten(),
            credential_set: connection.has_credential(),
        }
    }
}
