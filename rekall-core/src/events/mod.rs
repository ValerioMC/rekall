//! The in-process event bus. Services record events on their transaction (`Tx::publish`); the
//! transaction hands them to the bus once it commits. A domain owns its event type and implements
//! `StreamEvent` for it, so this module imports no domain.

mod domain_event;
mod event_bus;

pub use domain_event::{DomainEvent, PublishedEvent, StreamEvent};
pub use event_bus::EventBus;
