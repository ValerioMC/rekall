use rekall_model::{document, DocumentContextMode};

/// A note as a context hands it over: in full, or as a reference a session loads by its anchor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentView {
    pub title: String,
    pub kind: String,
    pub body_markdown: String,
    pub anchor: String,
    pub context_mode: DocumentContextMode,
}

impl DocumentView {
    pub fn of(document: &document::Model) -> Self {
        Self {
            title: document.title.clone(),
            kind: document.kind.clone(),
            body_markdown: document.body_markdown.clone(),
            anchor: document.anchor(),
            context_mode: document.context_mode,
        }
    }

    /// The same note, handed over in full whatever its mode: what loading it by its own anchor gives.
    pub fn in_full(mut self) -> Self {
        self.context_mode = DocumentContextMode::Full;
        self
    }
}
