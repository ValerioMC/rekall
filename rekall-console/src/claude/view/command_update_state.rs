use serde::Serialize;

/// Where the startup update of the installed `/rk` command files stands.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CommandUpdateState {
    Idle,
    Updating,
    Updated,
    Failed { reason: String },
}
