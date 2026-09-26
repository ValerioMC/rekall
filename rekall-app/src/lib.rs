//! The Rekall server: every module composed into one Axum application on one port, with what
//! only the application itself does — which database to open (`bootstrap`), backups and restores
//! (`backup`), the local-access guard (`security`), the console's static bundle (`spa`), the
//! health endpoint (`actuator`), restarting onto another database (`restart`, `server`), and
//! importing a database from the Java build's H2 file (`h2`).
//!
//! `rekall-server` is this as a plain binary; the desktop shell (`rekall-app/desktop`) runs the
//! same `server::start` in-process and points its window at it.

pub mod actuator;
pub mod backup;
pub mod bootstrap;
pub mod config;
pub mod h2;
pub mod methods;
pub mod restart;
pub mod security;
pub mod server;
pub mod spa;

pub use config::{AppConfig, Properties};
pub use h2::migrate_from_h2;
pub use server::{start, Running, StartOptions};
