use serde::Deserialize;

/// A diagram to create or replace. `sheets` is the document's sheets as sent; the service reads
/// and validates them.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DiagramRequest {
    pub project_id: Option<String>,
    pub task_id: Option<String>,
    pub title: Option<String>,
    pub question: Option<String>,
    pub sheets: Option<serde_json::Value>,
}
