use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeLaunch {
    #[serde(default)]
    pub(super) directory: String,
    #[serde(default)]
    pub(super) anchors: String,
    #[serde(default)]
    pub(super) skip_permissions: bool,
}
