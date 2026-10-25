use serde::Deserialize;

/// Whether the console leaves the task unreleased when it runs it.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TaskReleaseRequest {
    pub skipped: Option<bool>,
}
