//! The application: every module's router over one set of services on one database, served on
//! one port, and the supervisor that brings it down and back up in the same process when the
//! database changes (`ApplicationRestarter`).
//!
//! Closing follows Spring's order: the closing hooks run first (the run queue stops deciding, the
//! usage fetch and the event stream let go, every terminal is killed, the backup schedule stops),
//! then the web server stops taking requests and waits a bounded time for the ones in flight,
//! then the database pool closes.

mod instance;
mod running;
mod server_startup;
mod start_options;

pub use instance::Instance;
pub use running::Running;
pub use server_startup::{start, open_database, STOP_WINDOW};
pub use start_options::StartOptions;
