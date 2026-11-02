use rekall_kernel::Id;
use rekall_console::connection::ConnectionCompanyView;
use serde::Serialize;

/// A company a connection is shared with.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionCompanyResponse {
    pub id: Id,
    pub name: String,
}

impl From<ConnectionCompanyView> for ConnectionCompanyResponse {
    fn from(view: ConnectionCompanyView) -> Self {
        Self { id: view.id, name: view.name }
    }
}
