use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GuardProtectionState {
    Installed,
    NotInstalled,
    InvalidSettings,
}

/// Whether Claude Code runs the guard hooks. `problem` says why the settings file cannot be edited.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardProtection {
    pub state: GuardProtectionState,
    pub settings_file: String,
    pub problem: Option<String>,
}
