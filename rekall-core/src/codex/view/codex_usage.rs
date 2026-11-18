use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(tag = "status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CodexUsage {
    CliMissing,
    Unavailable,
    Ok { primary: Option<CodexUsageWindow>, secondary: Option<CodexUsageWindow> },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CodexUsageWindow {
    pub used_percent: f64,
    pub resets_at: Option<i64>,
    pub window_duration_mins: Option<i64>,
}
