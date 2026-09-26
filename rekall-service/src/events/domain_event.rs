use serde::Serialize;

use crate::commit::CommitReferenceStreamEvent;
use crate::note::NoteStreamEvent;
use crate::review::TaskReviewEvent;
use crate::step::StepStreamEvent;
use crate::wrapup::WrapupStreamEvent;

#[derive(Clone, Debug)]
pub enum DomainEvent {
    Steps(StepStreamEvent),
    TaskReview(TaskReviewEvent),
    Wrapup(WrapupStreamEvent),
    CommitReference(CommitReferenceStreamEvent),
    Note(NoteStreamEvent),
    /// A named frame from a module this one cannot see (the run queue's `run-queue`), already
    /// serialised; the caller has already waited for its commit.
    Broadcast { name: String, payload: serde_json::Value },
}

impl DomainEvent {
    /// The SSE frame name the console listens for.
    pub fn name(&self) -> &str {
        match self {
            Self::Steps(_) => "steps",
            Self::TaskReview(_) => "task-review",
            Self::Wrapup(_) => "wrapup",
            Self::CommitReference(_) => "commit-reference",
            Self::Note(_) => "note",
            Self::Broadcast { name, .. } => name,
        }
    }

    pub fn payload(&self) -> serde_json::Value {
        fn json(value: &impl Serialize) -> serde_json::Value {
            serde_json::to_value(value).unwrap_or(serde_json::Value::Null)
        }
        match self {
            Self::Steps(event) => json(event),
            Self::TaskReview(event) => json(event),
            Self::Wrapup(event) => json(event),
            Self::CommitReference(event) => json(event),
            Self::Note(event) => json(event),
            Self::Broadcast { payload, .. } => payload.clone(),
        }
    }
}
