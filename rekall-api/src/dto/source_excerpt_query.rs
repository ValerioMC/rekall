use serde::Deserialize;

/// A span of a file in a diagram's project folder. Lines are one-based; both are optional.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceExcerptQuery {
    pub file: Option<String>,
    pub start: Option<u32>,
    pub end: Option<u32>,
}
