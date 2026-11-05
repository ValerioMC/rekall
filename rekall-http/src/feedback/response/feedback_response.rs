use rekall_console::feedback::FeedbackView;
use rekall_kernel::{Id, Instant};
use rekall_store::{FeedbackKind, FeedbackState};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackResponse {
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
}

impl From<FeedbackView> for FeedbackResponse {
    fn from(view: FeedbackView) -> Self {
        Self {
            id: view.id,
            task_id: view.task_id,
            seq: view.seq,
            kind: view.kind,
            state: view.state,
            quoted_markdown: view.quoted_markdown,
            body_markdown: view.body_markdown,
            reply_markdown: view.reply_markdown,
            created_at: view.created_at,
            applied_at: view.applied_at,
        }
    }
}
