use serde::Deserialize;

/// A diagram to create or replace. `graph` is the Semantic Graph document as sent; the service
/// reads and validates it.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DiagramRequest {
    pub project_id: Option<String>,
    pub task_id: Option<String>,
    pub title: Option<String>,
    pub question: Option<String>,
    pub graph: Option<serde_json::Value>,
}
