use rekall_service::step::TaskStepView;
use serde::Serialize;

/// What running the task laid: the release graph's title, or none when nothing was laid, and the
/// task's whole checklist either way.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseOnRunResponse {
    pub laid: Option<String>,
    pub steps: Vec<TaskStepView>,
}
