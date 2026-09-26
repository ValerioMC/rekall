use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseView {
    pub id: String,
    pub label: String,
    pub path: String,
    pub active: bool,
    pub reachable: bool,
    pub added_at: String,
    pub last_used_at: String,
}
