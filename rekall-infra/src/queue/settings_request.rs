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
