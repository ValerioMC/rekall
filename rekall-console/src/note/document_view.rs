use rekall_common::{Id, Instant};
use rekall_model::document;
use rekall_model::DocumentContextMode;
use rekall_model::note_scope::NoteScope;

use crate::catalog::TaskRefView;

#[derive(Clone, Debug)]
pub struct DocumentView {
    pub id: Id,
    pub title: String,
    pub kind: String,
    pub body_markdown: String,
    pub tasks: Vec<TaskRefView>,
    pub context_mode: DocumentContextMode,
    pub scope: NoteScope,
    pub anchor: String,
    pub updated_at: Instant,
}

impl DocumentView {
    pub fn of(document: &document::Model, tasks: Vec<TaskRefView>) -> Self {
        Self {
            id: document.id,
            title: document.title.clone(),
            kind: document.kind.clone(),
            body_markdown: document.body_markdown.clone(),
            tasks,
            context_mode: document.context_mode,
            scope: document.scope(),
            anchor: document.anchor(),
            updated_at: document.updated_at,
        }
    }
}
