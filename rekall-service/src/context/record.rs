use rekall_model::{commit_reference, document, task_step, DocumentContextMode};

use crate::step::TaskStepView;
use crate::wrapup::WrapupView;

/// One record of a loaded context, materialised so it renders with no database behind it.
///
/// `fields` keeps the order the fields were added in. The Java record copied them into
/// `Map.copyOf`, whose iteration order is unspecified (and differs between JVM runs); insertion
/// order is the one stable order among those it could print, and the one the code wrote them in.
#[derive(Clone, Debug, Default)]
pub struct ContextRecord {
    pub kind: String,
    pub label: String,
    pub anchor: String,
    pub fields: Vec<(String, String)>,
    pub references: Vec<ContextRecord>,
    pub related: Vec<String>,
    pub documents: Vec<DocumentView>,
    pub steps: Vec<TaskStepView>,
    pub commits: Vec<ContextCommitView>,
    pub wrapup: Option<WrapupView>,
    pub blueprint: Option<String>,
    pub description: Option<String>,
}

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

/// One logged commit a task hands to `rekall_context`: the hash, the subject it was committed
/// with, the step it was logged against, and the diff it introduced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextCommitView {
    pub commit_hash: String,
    pub comment: String,
    pub step_title: Option<String>,
    pub diff: Option<String>,
}

impl ContextCommitView {
    pub fn of(reference: &commit_reference::Model, step: Option<&task_step::Model>) -> Self {
        Self {
            commit_hash: reference.commit_hash.clone(),
            comment: reference.comment.clone(),
            step_title: step.map(|s| s.title.clone()),
            diff: reference.diff.clone(),
        }
    }
}
