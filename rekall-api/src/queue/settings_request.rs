use rekall_console::queue::RunQueueSettingsCommand;
use serde::Deserialize;

/// `ceilingPercent` null is no ceiling; `model` and `effort` blank are the account's.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SettingsRequest {
    pub ceiling_percent: Option<i32>,
    pub skip_permissions: Option<bool>,
    pub model: Option<String>,
    pub effort: Option<String>,
}

impl From<SettingsRequest> for RunQueueSettingsCommand {
    fn from(request: SettingsRequest) -> Self {
        Self { ceiling_percent: request.ceiling_percent, skip_permissions: request.skip_permissions == Some(true), model: request.model, effort: request.effort }
    }
}
