//! The rules an agent hook applies before a tool runs and before a session stops. They decide
//! from the event and the facts the caller passes in, and touch no process, file or socket.

mod model;
mod port;
mod rule;
mod service;

pub use model::guard_context::GuardContext;
pub use model::guard_event::GuardEvent;
pub use model::guard_protection::{GuardProtection, GuardProtectionState};
pub use port::guard_installer::GuardInstaller;
pub use model::guard_running_step::GuardRunningStep;
pub use model::guard_scope::{GuardScope, GUARD_PORT_ENV, GUARD_SCOPE_ENV};
pub use model::guard_verdict::GuardVerdict;
pub use service::guard_service::GuardService;
