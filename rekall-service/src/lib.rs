//! The business logic. Everything the console and the MCP tools can do to the data goes through
//! a service here, and every service method that was `@Transactional` in Java opens (or joins) a
//! transaction here too, explicitly: a public method opens one, and its `*_in` twin runs inside a
//! transaction a caller already holds, which is how Spring's default propagation joined them.
//!
//! Events work as `@TransactionalEventListener(AFTER_COMMIT)` did: a service records them on the
//! transaction it runs in, and they reach the bus only once that transaction commits. A write
//! that rolls back announces nothing.

pub mod claude;
pub mod commit;
pub mod context;
pub mod events;
pub mod note;
pub mod review;
pub mod revision;
pub mod search;
pub mod step;
pub mod timeentry;
pub mod tx;
pub mod wrapup;

mod ctx;
mod load;
mod services;

pub use ctx::{system_clock, Clock, Ctx};
pub use events::{DomainEvent, EventBus};
pub use services::Services;
pub use tx::Tx;
