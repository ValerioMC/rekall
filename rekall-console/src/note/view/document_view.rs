use rekall_kernel::{Id, Instant};
use rekall_store::document;
use rekall_store::DocumentContextMode;

#[derive(Clone, Debug)]
pub struct DocumentView {
    pub id: Id,
    pub task_id: Id,
    pub title: String,
    pub kind: String,
    pub body_markdown: String,
    pub context_mode: DocumentContextMode,
    pub anchor: String,
    pub updated_at: Instant,
}

impl DocumentView {
    pub fn of(document: &document::Model) -> Self {
        Self {
            id: document.id,
            task_id: document.task_id,
            title: document.title.clone(),
            kind: document.kind.clone(),
            body_markdown: document.body_markdown.clone(),
            context_mode: document.context_mode,
            anchor: document.anchor(),
            updated_at: document.updated_at,
        }
    }
}
