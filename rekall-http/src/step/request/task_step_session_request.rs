use serde::Deserialize;

/// Whether a step opens a fresh session; false joins the one running.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TaskStepSessionRequest {
    pub starts_session: Option<bool>,
}
