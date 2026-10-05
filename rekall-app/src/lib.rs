//! The Rekall server: every module composed into one Axum application on one port, with what
//! only the application itself does — which database to open (`bootstrap`), backups and restores
//! (`backup`), the local-access guard (`security`), the console's static bundle (`spa`), the
//! health endpoint (`actuator`), restarting onto another database (`restart`, `server`), the
//! running version and the check for a newer release (`version`).
//!
//! `rekall-server` is this as a plain binary; the desktop shell (`rekall-app/desktop`) runs the
//! same `server::start` in-process and points its window at it.

pub mod actuator;
pub mod backup;
pub mod bootstrap;
pub mod config;
pub mod restart;
pub mod security;
pub mod server;
pub mod spa;
pub mod version;

pub use config::{AppConfig, Properties};
pub use server::{start, Running, StartOptions};
