use rekall_console::guard::GuardContextView;
use serde::{Deserialize, Serialize};

use super::running_step_response::RunningStepResponse;

/// Read by `rekall guard` as well as served, so one type holds both ends of the wire.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardContextResponse {
    pub project_anchor: Option<String>,
    pub auto_commit: bool,
    pub running_step: Option<RunningStepResponse>,
    pub protected_credential_references: Vec<String>,
}

impl From<GuardContextView> for GuardContextResponse {
    fn from(view: GuardContextView) -> Self {
        Self {
            project_anchor: view.project_anchor,
            auto_commit: view.auto_commit,
            running_step: view.running_step.map(Into::into),
            protected_credential_references: view.protected_credential_references,
        }
    }
}
