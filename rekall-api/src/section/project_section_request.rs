use serde::Deserialize;

/// A section to create or edit. On an edit, a field left out keeps what the section has.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProjectSectionRequest {
    pub title: Option<String>,
    pub body_markdown: Option<String>,
}
