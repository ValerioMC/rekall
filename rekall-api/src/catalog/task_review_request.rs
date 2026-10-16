use rekall_model::TaskStepState;
use serde::Deserialize;

/// Accept (`DONE`) or send back (`OPEN`) a stepless task; other states are refused.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TaskReviewRequest {
    pub review_state: Option<TaskStepState>,
    pub note: Option<String>,
}
