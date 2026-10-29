//! The backups of the open database, downloading one and putting one back, served by
//! `rekall_console::backup`.

mod controller;
mod response;
mod routes;

pub(crate) use routes::routes;

use response::restore_started::RestoreStarted;
