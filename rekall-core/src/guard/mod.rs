//! The rules a Claude Code hook applies before a tool runs and before a session stops. They decide
//! from the event and the facts the caller passes in, and touch no process, file or socket.

mod model;
mod rule;
mod service;

pub use model::guard_context::GuardContext;
pub use model::guard_event::GuardEvent;
pub use model::guard_verdict::GuardVerdict;
pub use service::guard_service::GuardService;
