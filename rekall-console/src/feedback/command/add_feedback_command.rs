use rekall_store::FeedbackKind;

/// Feedback the person writes under the description: on the work (`IMPLEMENTATION`) or on the plan
/// (`PLAN`). A revision is never typed: it comes from saving the description.
#[derive(Clone, Debug)]
pub struct AddFeedbackCommand {
    pub kind: FeedbackKind,
    pub body: String,
}
