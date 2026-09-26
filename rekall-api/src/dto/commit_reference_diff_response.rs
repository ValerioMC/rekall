use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitReferenceDiffResponse {
    pub diff: Option<String>,
}
