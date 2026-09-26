use serde::Serialize;

use super::DatabaseView;

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: &'static str,
    pub active: Option<DatabaseView>,
    pub databases: Vec<DatabaseView>,
}
