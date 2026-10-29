//! The Rekall server: every module composed into one Axum application on one port. It holds only
//! what the application itself does (the local-access guard `security`, the console's static
//! bundle `spa`, the health endpoint `actuator`, the supervisor in `server`) and the `AppConfig`
//! that gathers every module's settings. The rest is in `rekall-infra`.
//!
//! `rekall-server` is this as a plain binary; the desktop shell (`rekall-app/desktop`) runs the
//! same `server::start` in-process and points its window at it.

pub mod actuator;
pub mod config;
pub mod security;
pub mod server;
pub mod spa;

pub use config::AppConfig;
pub use rekall_infra::config::Properties;
pub use server::{start, Running, StartOptions};
