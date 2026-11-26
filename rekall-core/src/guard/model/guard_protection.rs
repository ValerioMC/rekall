use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GuardProtectionState {
    Installed,
    NotInstalled,
    NeedsRepair,
    InvalidSettings,
    Unsupported,
    TrustRequired,
    Disabled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardProtection {
    pub state: GuardProtectionState,
    pub settings_file: String,
    pub configured: bool,
    pub problem: Option<String>,
}
