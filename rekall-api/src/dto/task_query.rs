use serde::Deserialize;

/// The optional project filter of the task list.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskQuery {
    pub project_id: Option<String>,
}
