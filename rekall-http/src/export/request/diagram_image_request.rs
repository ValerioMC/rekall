use serde::Deserialize;

/// The picture of one mermaid fence: its source and the PNG as base64.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DiagramImageRequest {
    pub source: Option<String>,
    pub png: Option<String>,
}
