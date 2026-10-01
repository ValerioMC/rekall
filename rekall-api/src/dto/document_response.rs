use rekall_common::{Id, Instant};
use rekall_model::document;
use rekall_model::note_scope::NoteScope;
use rekall_model::DocumentContextMode;
use serde::Serialize;

use super::TaskRef;

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

impl DocumentResponse {
    pub fn of(document: &document::Model, tasks: Vec<TaskRef>) -> Self {
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
