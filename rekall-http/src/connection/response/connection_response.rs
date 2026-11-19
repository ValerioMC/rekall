use rekall_kernel::{Id, Instant};
use rekall_console::connection::ConnectionView;
use rekall_store::ConnectionKind;
use serde::Serialize;

/// A connection as the console shows it. Never the credential: only when one was stored.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionResponse {
    pub id: Id,
    pub kind: ConnectionKind,
    pub label: String,
    pub title: String,
    pub base_url: Option<String>,
    pub kube_context: Option<String>,
    pub proxy_url: Option<String>,
    pub accept_self_signed: bool,
    pub root_path: Option<String>,
    pub company_ids: Vec<Id>,
    pub credential_set_at: Option<Instant>,
    pub project_count: usize,
    pub updated_at: Instant,
}

impl From<ConnectionView> for ConnectionResponse {
    fn from(view: ConnectionView) -> Self {
        Self {
            id: view.id,
            kind: view.kind,
            label: view.label,
            title: view.title,
            base_url: view.base_url,
            kube_context: view.kube_context,
            proxy_url: view.proxy_url,
            accept_self_signed: view.accept_self_signed,
            root_path: view.root_path,
            company_ids: view.company_ids,
            credential_set_at: view.credential_set_at,
            project_count: view.project_count,
            updated_at: view.updated_at,
        }
    }
}
