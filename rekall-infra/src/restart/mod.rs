//! `ApplicationRestarter`: switching or restoring a database restarts the application in the same
//! process, so the next start opens what the registry now names. The supervisor in `crate::server`
//! owns the loop; handlers hold a `Restarter` that asks it to go round once more.

mod restart_request;
mod restarter;

pub use restart_request::RestartRequest;
pub use restarter::{Hook, RESTART_DELAY, Restarter, run_hook, announce};
