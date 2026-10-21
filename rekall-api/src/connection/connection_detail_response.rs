use rekall_model::{external_connection, project, project_connection};
use serde::Serialize;

use super::{ConnectionProjectResponse, ConnectionResponse};

/// One connection with the projects that bind it.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionDetailResponse {
    #[serde(flatten)]
    pub connection: ConnectionResponse,
    pub projects: Vec<ConnectionProjectResponse>,
}

impl ConnectionDetailResponse {
    pub fn of(connection: &external_connection::Model, projects: &[(project_connection::Model, project::Model)]) -> Self {
        Self {
            connection: ConnectionResponse::of(connection, projects.len()),
            projects: projects.iter().map(|(binding, project)| ConnectionProjectResponse::of(binding, project)).collect(),
        }
    }
}
