use rekall_store::{task_feedback, FeedbackKind, FeedbackState};

/// A thread entry as a context hands it over.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeedbackView {
    pub seq: i32,
    pub kind: FeedbackKind,
    pub state: FeedbackState,
    pub quoted_markdown: Option<String>,
    pub body_markdown: String,
    pub reply_markdown: Option<String>,
}

impl FeedbackView {
    pub fn of(entry: &task_feedback::Model) -> Self {
        Self {
            seq: entry.seq,
            kind: entry.kind,
            state: entry.state,
            quoted_markdown: entry.quoted_markdown.clone(),
            body_markdown: entry.body_markdown.clone(),
            reply_markdown: entry.reply_markdown.clone(),
        }
    }
}
