use rekall_common::{jstr, Id, RekallError};
use rekall_diagram::DiagramKind;
use serde::Deserialize;

use super::{TerminalMode, TerminalModeName};

/// What the console sends to open a terminal. `stepId`, `model` and `effort` are optional; an
/// unknown or blank model or effort leaves the account default. A missing `mode` is
/// [`TerminalMode::Work`]; `GENERATE` needs a `request`, and `kinds` names the diagram types.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OpenTerminalRequest {
    pub step_id: Option<Id>,
    pub skip_permissions: Option<bool>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub mode: Option<TerminalModeName>,
    pub request: Option<String>,
    pub kinds: Option<Vec<String>>,
}

impl OpenTerminalRequest {
    pub fn mode(&self) -> Result<TerminalMode, RekallError> {
        match self.mode {
            None | Some(TerminalModeName::Work) => Ok(TerminalMode::Work),
            Some(TerminalModeName::Plan) => Ok(TerminalMode::Plan),
            Some(TerminalModeName::Generate) => self.generate_mode(),
        }
    }

    fn generate_mode(&self) -> Result<TerminalMode, RekallError> {
        let Some(request) = self.request.as_deref().filter(|request| !jstr::is_blank(request)) else {
            return Err(RekallError::illegal("Say what the diagram should show before generating it."));
        };
        Ok(TerminalMode::Generate { request: request.to_string(), kinds: self.diagram_kinds()? })
    }

    /// The types asked for, in order and without repeats.
    fn diagram_kinds(&self) -> Result<Vec<DiagramKind>, RekallError> {
        let mut kinds: Vec<DiagramKind> = Vec::new();
        for name in self.kinds.iter().flatten() {
            let kind = DiagramKind::of_name(name.trim()).ok_or_else(|| RekallError::illegal(format!("'{name}' is not a diagram type.")))?;
            if !kinds.contains(&kind) {
                kinds.push(kind);
            }
        }
        Ok(kinds)
    }
}
