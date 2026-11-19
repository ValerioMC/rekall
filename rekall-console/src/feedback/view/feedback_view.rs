use rekall_kernel::{Id, Instant};
use rekall_store::{task_feedback, FeedbackKind, FeedbackState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeedbackView {
    pub id: Id,
    pub task_id: Id,
    pub seq: i32,
    pub kind: FeedbackKind,
    pub state: FeedbackState,
    pub quoted_markdown: Option<String>,
    pub body_markdown: String,
    pub reply_markdown: Option<String>,
    pub created_at: Instant,
    pub applied_at: Option<Instant>,
    pub pass: Option<i32>,
    pub taken_at: Option<Instant>,
}

impl FeedbackView {
    pub fn of(entry: &task_feedback::Model) -> Self {
        Self {
            id: entry.id,
            task_id: entry.task_id,
            seq: entry.seq,
            kind: entry.kind,
            state: entry.state,
            quoted_markdown: entry.quoted_markdown.clone(),
            body_markdown: entry.body_markdown.clone(),
            reply_markdown: entry.reply_markdown.clone(),
            created_at: entry.created_at,
            applied_at: entry.applied_at,
            pass: entry.pass,
            taken_at: entry.taken_at,
        }
    }
}
