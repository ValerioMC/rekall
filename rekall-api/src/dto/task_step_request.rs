use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TaskStepRequest {
    pub title: Option<String>,
    pub body_markdown: Option<String>,
}
