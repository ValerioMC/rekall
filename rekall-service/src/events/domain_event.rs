use serde::Serialize;

use crate::commit::CommitReferenceStreamEvent;
use crate::diagram::DiagramStreamEvent;
use crate::note::NoteStreamEvent;
use crate::review::TaskReviewEvent;
use crate::section::SectionStreamEvent;
use crate::step::StepStreamEvent;
use crate::task::TaskStreamEvent;
use crate::token::TokenUsageStreamEvent;
use crate::wrapup::WrapupStreamEvent;

#[derive(Clone, Debug)]
pub enum DomainEvent {
    Steps(StepStreamEvent),
    TaskCreated(TaskStreamEvent),
    TaskReview(TaskReviewEvent),
    Wrapup(WrapupStreamEvent),
    CommitReference(CommitReferenceStreamEvent),
    Note(NoteStreamEvent),
    Diagram(DiagramStreamEvent),
    ProjectSection(SectionStreamEvent),
    TokenUsage(TokenUsageStreamEvent),
    /// A named frame from a module this one cannot see (the run queue's `run-queue`), already
    /// serialised; the caller has already waited for its commit.
    Broadcast { name: String, payload: serde_json::Value },
}

impl DomainEvent {
    /// The SSE frame name the console listens for.
    pub fn name(&self) -> &str {
        match self {
            Self::Steps(_) => "steps",
            Self::TaskCreated(_) => "task-created",
            Self::TaskReview(_) => "task-review",
            Self::Wrapup(_) => "wrapup",
            Self::CommitReference(_) => "commit-reference",
            Self::Note(_) => "note",
            Self::Diagram(_) => "diagram",
            Self::ProjectSection(_) => "project-section",
            Self::TokenUsage(_) => "token-usage",
            Self::Broadcast { name, .. } => name,
        }
    }

    pub fn payload(&self) -> serde_json::Value {
        fn json(value: &impl Serialize) -> serde_json::Value {
            serde_json::to_value(value).unwrap_or(serde_json::Value::Null)
        }
        match self {
            Self::Steps(event) => json(event),
            Self::TaskCreated(event) => json(event),
            Self::TaskReview(event) => json(event),
            Self::Wrapup(event) => json(event),
            Self::CommitReference(event) => json(event),
            Self::Note(event) => json(event),
            Self::Diagram(event) => json(event),
            Self::ProjectSection(event) => json(event),
            Self::TokenUsage(event) => json(event),
            Self::Broadcast { payload, .. } => payload.clone(),
        }
    }
}
