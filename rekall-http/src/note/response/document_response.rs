use rekall_kernel::{Id, Instant};
use rekall_console::note::DocumentView;
use rekall_store::DocumentContextMode;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentResponse {
    pub id: Id,
    pub task_id: Id,
    pub title: String,
    pub kind: String,
    pub body_markdown: String,
    pub context_mode: DocumentContextMode,
    pub anchor: String,
    pub updated_at: Instant,
}

impl From<DocumentView> for DocumentResponse {
    fn from(view: DocumentView) -> Self {
        Self {
            id: view.id,
            task_id: view.task_id,
            title: view.title,
            kind: view.kind,
            body_markdown: view.body_markdown,
            context_mode: view.context_mode,
            anchor: view.anchor,
            updated_at: view.updated_at,
        }
    }
}
