use rekall_console::connection::ConnectionDetailView;
use serde::Serialize;

use crate::connection::{ConnectionCompanyResponse, ConnectionProjectResponse, ConnectionResponse};

/// One connection with the companies it is shared with and the projects that bind it, a row per binding.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionDetailResponse {
    #[serde(flatten)]
    pub connection: ConnectionResponse,
    pub companies: Vec<ConnectionCompanyResponse>,
    pub projects: Vec<ConnectionProjectResponse>,
}

impl From<ConnectionDetailView> for ConnectionDetailResponse {
    fn from(view: ConnectionDetailView) -> Self {
        Self {
            connection: view.connection.into(),
            companies: view.companies.into_iter().map(ConnectionCompanyResponse::from).collect(),
            projects: view.projects.into_iter().map(ConnectionProjectResponse::from).collect(),
        }
    }
}
