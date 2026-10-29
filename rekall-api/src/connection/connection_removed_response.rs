use rekall_console::connection::ConnectionRemovedView;
use serde::Serialize;

/// What deleting a connection took with it.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionRemovedResponse {
    pub unbound_projects: usize,
}

impl From<ConnectionRemovedView> for ConnectionRemovedResponse {
    fn from(view: ConnectionRemovedView) -> Self {
        Self { unbound_projects: view.unbound_projects }
    }
}
