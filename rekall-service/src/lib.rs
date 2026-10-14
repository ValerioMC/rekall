//! The business logic. Everything the console and the MCP tools can do to the data goes through
//! a service here. A public method opens its own transaction (`in_read!` / `in_write!`); its
//! `*_in` twin runs inside one a caller already holds, so two services can share one commit.
//!
//! A service records events on the transaction it runs in, and they reach the bus only once that
//! transaction commits. A write that rolls back announces nothing.

pub mod claude;
pub mod commit;
pub mod component;
pub mod context;
pub mod diagram;
pub mod events;
pub mod flow;
pub mod note;
pub mod review;
pub mod revision;
pub mod search;
pub mod section;
pub mod step;
pub mod task;
pub mod timeentry;
pub mod token;
pub mod tx;
pub mod wrapup;

mod ctx;
mod load;
mod services;
mod text_rule;

pub use ctx::{system_clock, Clock, Ctx};
pub use events::{DomainEvent, EventBus};
pub use services::Services;
pub use tx::Tx;
