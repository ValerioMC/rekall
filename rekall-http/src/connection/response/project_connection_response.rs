use rekall_kernel::{Id, Instant};
use rekall_console::connection::ProjectConnectionView;
use rekall_store::ConnectionKind;
use serde::Serialize;

/// A connection a project binds, as the project page shows it.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConnectionResponse {
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

impl From<ProjectConnectionView> for ProjectConnectionResponse {
    fn from(view: ProjectConnectionView) -> Self {
        Self {
            id: view.id,
            connection_id: view.connection_id,
            project_id: view.project_id,
            kind: view.kind,
            label: view.label,
            title: view.title,
            base_url: view.base_url,
            kube_context: view.kube_context,
            target: view.target,
            credential_set_at: view.credential_set_at,
        }
    }
}
