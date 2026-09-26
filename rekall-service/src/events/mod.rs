//! The in-process event bus: what `ApplicationEventPublisher` carried. Services record events on
//! their transaction (`Tx::publish`); the transaction hands them to the bus once it commits.

mod domain_event;
mod event_bus;

pub use domain_event::DomainEvent;
pub use event_bus::EventBus;
