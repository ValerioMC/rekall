//! The backups of the open database, downloading one and putting one back, served by
//! `rekall_console::backup`.

mod backup_controller;
mod restore_started;
mod routes;

pub(crate) use routes::routes;

use restore_started::RestoreStarted;
