use rekall_store::FeedbackKind;
use serde::Deserialize;

/// Feedback on the work (`IMPLEMENTATION`) or on the plan (`PLAN`); a revision is never sent.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AddFeedbackRequest {
    pub kind: Option<FeedbackKind>,
    pub body_markdown: Option<String>,
}
