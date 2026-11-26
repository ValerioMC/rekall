use rekall_infra::session::SessionProvider;
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
    #[serde(default)]
    pub(super) provider: SessionProvider,
    pub(super) task_id: Option<uuid::Uuid>,
    pub(super) model: Option<String>,
    pub(super) effort: Option<String>,
}
