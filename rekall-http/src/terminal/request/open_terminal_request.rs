use rekall_console::terminal::OpenTerminalCommand;
use rekall_core::claude::{TerminalMode, TerminalTarget};
use rekall_core::vulnerability::rule::rules;
use rekall_diagram::DiagramKind;
use rekall_kernel::{jstr, Id, RekallError};
use serde::Deserialize;

use crate::terminal::TerminalModeName;

/// What the console sends to open a terminal. `stepId`, `model` and `effort` are optional; an
/// unknown or blank model or effort leaves the account default. A missing `mode` is
/// [`TerminalMode::Work`]; `PLAN_FLOW` asks for a flow rather than a checklist; `GENERATE` and
/// `EXPLAIN` need a `request`, and `kinds` names the diagram types a generation draws; `ASSESS`
/// needs the GHSA id of the `advisory` to judge, and `ASSESS_ALL` judges every one not judged yet.
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
    pub advisory: Option<String>,
}

impl OpenTerminalRequest {
    /// The command to open a terminal on `target`, once the mode the request names is read.
    pub fn command(self, target: TerminalTarget) -> Result<OpenTerminalCommand, RekallError> {
        let mode = self.mode()?;
        Ok(OpenTerminalCommand { target, step_id: self.step_id, skip_permissions: self.skip_permissions.unwrap_or(false), model: self.model, effort: self.effort, mode })
    }

    pub fn mode(&self) -> Result<TerminalMode, RekallError> {
        match self.mode {
            None | Some(TerminalModeName::Work) => Ok(TerminalMode::Work),
            Some(TerminalModeName::Plan) => Ok(TerminalMode::Plan),
            Some(TerminalModeName::PlanFlow) => Ok(TerminalMode::PlanFlow),
            Some(TerminalModeName::Generate) => Ok(TerminalMode::Generate { request: self.diagram_request()?, kinds: self.diagram_kinds()? }),
            Some(TerminalModeName::Explain) => Ok(TerminalMode::Explain { request: self.diagram_request()? }),
            Some(TerminalModeName::Inventory) => Ok(TerminalMode::Inventory),
            Some(TerminalModeName::Assess) => Ok(TerminalMode::Assess { advisory: rules::ghsa_id(self.advisory.as_deref())? }),
            Some(TerminalModeName::AssessAll) => Ok(TerminalMode::AssessAll),
        }
    }

    fn diagram_request(&self) -> Result<String, RekallError> {
        match self.request.as_deref().filter(|request| !jstr::is_blank(request)) {
            Some(request) => Ok(request.to_string()),
            None => Err(RekallError::illegal("Say what the diagram should show before generating it.")),
        }
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

#[cfg(test)]
#[path = "../../../tests/unit/terminal/request/open_terminal_request_tests.rs"]
mod tests;
