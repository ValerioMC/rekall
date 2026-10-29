use rekall_common::{Id, Instant};
use rekall_model::{external_connection, ConnectionKind};

/// A connection as the console shows it. Never the credential: only when one was stored.
#[derive(Clone, Debug)]
pub struct ConnectionView {
    pub id: Id,
    pub company_id: Id,
    pub kind: ConnectionKind,
    pub label: String,
    pub title: String,
    pub base_url: Option<String>,
    pub kube_context: Option<String>,
    pub proxy_url: Option<String>,
    pub credential_set_at: Option<Instant>,
    pub project_count: usize,
    pub updated_at: Instant,
}

impl ConnectionView {
    pub fn of(connection: &external_connection::Model, project_count: usize) -> Self {
        Self {
            id: connection.id,
            company_id: connection.company_id,
            kind: connection.kind,
            label: connection.label.clone(),
            title: connection.title.clone(),
            base_url: connection.base_url.clone(),
            kube_context: connection.kube_context.clone(),
            proxy_url: connection.proxy_url.clone(),
            credential_set_at: connection.secret_set_at,
            project_count,
            updated_at: connection.updated_at,
        }
    }
}
