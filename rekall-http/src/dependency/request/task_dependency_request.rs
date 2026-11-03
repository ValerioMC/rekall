use serde::Deserialize;

/// The task another one is to wait for.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TaskDependencyRequest {
    pub depends_on_task_id: Option<String>,
}
