use rekall_common::Id;
use rekall_model::note_scope::NoteScope;
use rekall_model::DocumentContextMode;
use serde::Deserialize;

/// `contextMode` left out keeps what the note had, or `FULL` for a new one. `scope` left out
/// keeps the note's, or for a new one takes the narrowest its tasks share.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DocumentRequest {
    pub title: Option<String>,
    pub kind: Option<String>,
    pub body_markdown: Option<String>,
    pub task_ids: Option<Vec<Id>>,
    pub context_mode: Option<DocumentContextMode>,
    pub scope: Option<NoteScope>,
}
