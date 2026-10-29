use rekall_common::{Id, Instant};
use rekall_store::{external_connection, project_connection, ConnectionKind};

/// A connection a project binds, as the project page shows it.
#[derive(Clone, Debug)]
pub struct ProjectConnectionView {
    pub id: Id,
    pub connection_id: Id,
    pub project_id: Id,
    pub kind: ConnectionKind,
    /// The binding's own label, which a session names it by.
    pub label: String,
    pub title: String,
    pub base_url: Option<String>,
    pub kube_context: Option<String>,
    pub target: Option<String>,
    pub credential_set_at: Option<Instant>,
}

impl ProjectConnectionView {
    pub fn of(binding: &project_connection::Model, connection: &external_connection::Model) -> Self {
        Self {
            id: binding.id,
            connection_id: connection.id,
            project_id: binding.project_id,
            kind: connection.kind,
            label: binding.label.clone(),
            title: connection.title.clone(),
            base_url: connection.base_url.clone(),
            kube_context: connection.kube_context.clone(),
            target: binding.target.clone(),
            credential_set_at: connection.secret_set_at,
        }
    }
}
