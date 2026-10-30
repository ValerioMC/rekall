use serde::Serialize;

/// The saved setting. The secret key never leaves the credential store: only whether one is saved.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetrySettingsView {
    pub enabled: bool,
    pub host: String,
    pub public_key: String,
    pub log_content: bool,
    pub secret_saved: bool,
}
