use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

use serde::Serialize;

/// What the bus carries: a frame name the console listens for and the payload it receives. A
/// domain owns its event type and never edits this module to add one.
pub trait DomainEvent: Debug + Send + Sync + 'static {
    /// The SSE frame name.
    fn name(&self) -> &str;

    fn payload(&self) -> serde_json::Value;

    fn as_any(&self) -> &dyn Any;
}

/// An event on its way through the bus; every subscriber shares the one value.
pub type PublishedEvent = Arc<dyn DomainEvent>;

impl dyn DomainEvent {
    /// The event as `E`, for a subscriber that acts on one kind of event.
    pub fn downcast<E: DomainEvent>(&self) -> Option<&E> {
        self.as_any().downcast_ref()
    }
}

/// A serialisable event with a fixed frame name: implementing this is all a domain does to
/// announce a change.
pub trait StreamEvent: Debug + Serialize + Send + Sync + 'static {
    const FRAME: &'static str;
}

impl<E: StreamEvent> DomainEvent for E {
    fn name(&self) -> &str {
        E::FRAME
    }

    fn payload(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
#[path = "../../tests/unit/events/domain_event_tests.rs"]
mod tests;
