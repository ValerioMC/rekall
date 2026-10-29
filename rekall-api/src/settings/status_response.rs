use rekall_console::settings::DatabaseFoldersView;
use serde::Serialize;

use super::DatabaseResponse;

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: &'static str,
    pub active: Option<DatabaseResponse>,
    pub databases: Vec<DatabaseResponse>,
}

impl From<DatabaseFoldersView> for StatusResponse {
    fn from(view: DatabaseFoldersView) -> Self {
        Self {
            status: view.status.as_str(),
            active: view.active.map(DatabaseResponse::from),
            databases: view.databases.into_iter().map(DatabaseResponse::from).collect(),
        }
    }
}
