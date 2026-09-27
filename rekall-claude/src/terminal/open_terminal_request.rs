use rekall_common::{jstr, Id, RekallError};
use serde::Deserialize;

use super::{TerminalMode, TerminalModeName};

/// What the console sends to open a terminal. `stepId`, `model` and `effort` are optional; an
/// unknown or blank model or effort leaves the account default. A missing `mode` is
/// [`TerminalMode::Work`]; `GENERATE` needs a `request`.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OpenTerminalRequest {
    pub step_id: Option<Id>,
    pub skip_permissions: Option<bool>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub mode: Option<TerminalModeName>,
    pub request: Option<String>,
}

impl OpenTerminalRequest {
    pub fn mode(&self) -> Result<TerminalMode, RekallError> {
        match self.mode {
            None | Some(TerminalModeName::Work) => Ok(TerminalMode::Work),
            Some(TerminalModeName::Plan) => Ok(TerminalMode::Plan),
            Some(TerminalModeName::Generate) => match self.request.as_deref().filter(|request| !jstr::is_blank(request)) {
                Some(request) => Ok(TerminalMode::Generate { request: request.to_string() }),
                None => Err(RekallError::illegal("Say what the diagram should show before generating it.")),
            },
        }
    }
}
