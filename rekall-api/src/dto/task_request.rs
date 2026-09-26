use rekall_common::Id;
use rekall_model::TaskStatus;
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TaskRequest {
    pub label: Option<String>,
    pub title: Option<String>,
    pub status: Option<TaskStatus>,
    pub description: Option<String>,
    pub project_id: Option<Id>,
    pub tag_id: Option<Id>,
}
