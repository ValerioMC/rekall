use rekall_console::connection::ConnectionDetailView;
use serde::Serialize;

use super::{ConnectionProjectResponse, ConnectionResponse};

/// One connection with the projects that bind it, a row per binding.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionDetailResponse {
    #[serde(flatten)]
    pub connection: ConnectionResponse,
    pub projects: Vec<ConnectionProjectResponse>,
}

impl From<ConnectionDetailView> for ConnectionDetailResponse {
    fn from(view: ConnectionDetailView) -> Self {
        Self { connection: view.connection.into(), projects: view.projects.into_iter().map(ConnectionProjectResponse::from).collect() }
    }
}
