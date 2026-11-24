use serde::Deserialize;

/// The task last seen in the project; left out or null for its task list.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RememberPlaceRequest {
    pub task_id: Option<String>,
}
