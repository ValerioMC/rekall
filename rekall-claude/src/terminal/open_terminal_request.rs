use rekall_common::Id;
use serde::Deserialize;

use super::TerminalMode;

/// What the console sends to open a terminal. `stepId`, `model` and `effort` are optional; an
/// unknown or blank model or effort leaves the account default. A missing `mode` is
/// [`TerminalMode::Work`].
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OpenTerminalRequest {
    pub step_id: Option<Id>,
    pub skip_permissions: Option<bool>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub mode: Option<TerminalMode>,
}

impl OpenTerminalRequest {
    pub fn mode_or_default(&self) -> TerminalMode {
        self.mode.unwrap_or_default()
    }
}
