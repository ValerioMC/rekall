use rekall_console::guard::RunningStepView;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunningStepResponse {
    pub number: u32,
    pub task_anchor: String,
}

impl From<RunningStepView> for RunningStepResponse {
    fn from(view: RunningStepView) -> Self {
        Self { number: view.number, task_anchor: view.task_anchor }
    }
}
