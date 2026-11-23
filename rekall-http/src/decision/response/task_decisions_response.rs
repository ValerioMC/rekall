use rekall_console::decision::TaskDecisionsView;
use serde::Serialize;

use super::decision_response::DecisionResponse;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDecisionsResponse {
    pub decisions: Vec<DecisionResponse>,
    pub session_block: String,
}

impl From<TaskDecisionsView> for TaskDecisionsResponse {
    fn from(view: TaskDecisionsView) -> Self {
        Self { decisions: view.decisions.into_iter().map(DecisionResponse::from).collect(), session_block: view.session_block }
    }
}
