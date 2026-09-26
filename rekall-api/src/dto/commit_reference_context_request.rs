use serde::Deserialize;

/// An absent key reads as off, not as a failed request.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CommitReferenceContextRequest {
    pub in_context: Option<bool>,
}
