use rekall_kernel::Id;
use rekall_console::note::DocumentCommand;
use rekall_store::DocumentContextMode;
use rekall_store::note_scope::NoteScope;
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

impl From<DocumentRequest> for DocumentCommand {
    fn from(request: DocumentRequest) -> Self {
        Self {
            title: request.title,
            kind: request.kind,
            body_markdown: request.body_markdown,
            task_ids: request.task_ids,
            context_mode: request.context_mode,
            scope: request.scope,
        }
    }
}
