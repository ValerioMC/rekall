use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TaskStepPatchRequest {
    pub title: Option<String>,
    pub body_markdown: Option<String>,
    pub done: Option<bool>,
    pub draft: Option<bool>,
}
