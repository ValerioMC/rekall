use rekall_common::{Id, Instant};
use rekall_model::{external_connection, project_connection, ConnectionKind};
use serde::Serialize;

/// A connection a project binds, as the project page shows it.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConnectionResponse {
    pub connection_id: Id,
    pub project_id: Id,
    pub kind: ConnectionKind,
    pub label: String,
    pub title: String,
    pub base_url: Option<String>,
    pub kube_context: Option<String>,
    pub target: Option<String>,
    pub credential_set_at: Option<Instant>,
}

impl ProjectConnectionResponse {
    pub fn of(binding: &project_connection::Model, connection: &external_connection::Model) -> Self {
        Self {
            connection_id: connection.id,
            project_id: binding.project_id,
            kind: connection.kind,
            label: connection.label.clone(),
            title: connection.title.clone(),
            base_url: connection.base_url.clone(),
            kube_context: connection.kube_context.clone(),
            target: binding.target.clone(),
            credential_set_at: connection.secret_set_at,
        }
    }
}
