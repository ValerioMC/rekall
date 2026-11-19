use rekall_store::FeedbackKind;
use serde::Deserialize;

/// What a waiting revision is applied to: the work (`IMPLEMENTATION`) or the plan (`PLAN`).
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RetargetFeedbackRequest {
    pub kind: Option<FeedbackKind>,
}
