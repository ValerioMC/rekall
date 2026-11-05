use rekall_store::FeedbackKind;

/// Feedback the person writes under the description: on the work (`IMPLEMENTATION`) or on the plan
/// (`PLAN`). A revision is never typed: it comes from saving the description. `follows_seq` adds
/// detail to an entry a session already applied.
#[derive(Clone, Debug)]
pub struct AddFeedbackCommand {
    pub kind: FeedbackKind,
    pub body: String,
    pub follows_seq: Option<i32>,
}
