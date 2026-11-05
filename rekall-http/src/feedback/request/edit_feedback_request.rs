use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EditFeedbackRequest {
    pub body_markdown: Option<String>,
}
