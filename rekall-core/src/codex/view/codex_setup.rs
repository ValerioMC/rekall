use serde::Serialize;

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CodexSetup {
    Connected,
    NotConnected,
    CliMissing,
}
