use rekall_core::telemetry::TelemetryChange;
use serde::Deserialize;

/// The setting as the form saves it. A secret key left out keeps the one stored. It has no
/// `Debug`, so the key cannot reach a log.
#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TelemetrySettingsRequest {
    pub enabled: bool,
    pub host: Option<String>,
    pub public_key: Option<String>,
    pub log_content: bool,
    pub secret_key: Option<String>,
}

impl From<TelemetrySettingsRequest> for TelemetryChange {
    fn from(request: TelemetrySettingsRequest) -> Self {
        Self { enabled: request.enabled, host: request.host, public_key: request.public_key, log_content: request.log_content, secret_key: request.secret_key }
    }
}
