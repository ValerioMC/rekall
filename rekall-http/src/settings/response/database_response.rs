use rekall_console::settings::DatabaseView;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseResponse {
    pub id: String,
    pub label: String,
    pub path: String,
    pub active: bool,
    pub reachable: bool,
    pub added_at: String,
    pub last_used_at: String,
}

impl From<DatabaseView> for DatabaseResponse {
    fn from(view: DatabaseView) -> Self {
        Self {
            id: view.id,
            label: view.label,
            path: view.path,
            active: view.active,
            reachable: view.reachable,
            added_at: view.added_at,
            last_used_at: view.last_used_at,
        }
    }
}
