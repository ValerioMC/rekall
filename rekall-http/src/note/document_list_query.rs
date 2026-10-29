use serde::Deserialize;

/// Which task or project the note list is narrowed to.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentListQuery {
    pub task_id: Option<String>,
    pub project_id: Option<String>,
}
