//! The in-process event bus: what `ApplicationEventPublisher` carried. Services record events on
//! their transaction (`Tx::publish`); the transaction hands them to the bus once it commits.

use serde::Serialize;
use tokio::sync::broadcast;

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

#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<DomainEvent>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(4096);
        Self { sender }
    }

    pub fn publish(&self, event: DomainEvent) {
        // No subscriber is not an error: nobody is watching yet.
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DomainEvent> {
        self.sender.subscribe()
    }
}
