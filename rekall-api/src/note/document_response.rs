use rekall_common::{Id, Instant};
use rekall_console::note::DocumentView;
use rekall_model::DocumentContextMode;
use rekall_model::note_scope::NoteScope;
use serde::Serialize;

use crate::catalog::TaskRef;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentResponse {
    pub id: Id,
    pub title: String,
    pub kind: String,
    pub body_markdown: String,
    pub tasks: Vec<TaskRef>,
    pub context_mode: DocumentContextMode,
    pub scope: NoteScope,
    pub anchor: String,
    pub updated_at: Instant,
}

impl From<DocumentView> for DocumentResponse {
    fn from(view: DocumentView) -> Self {
        Self {
            id: view.id,
            title: view.title,
            kind: view.kind,
            body_markdown: view.body_markdown,
            tasks: view.tasks.into_iter().map(TaskRef::from).collect(),
            context_mode: view.context_mode,
            scope: view.scope,
            anchor: view.anchor,
            updated_at: view.updated_at,
        }
    }
}
