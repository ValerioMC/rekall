//! Rotating copies of the open database in a `backups` folder beside it, and putting one back.
//!
//! H2 wrote a backup with `BACKUP TO`; SQLite writes a consistent copy of a live database with
//! `VACUUM INTO`, which this zips under the database file's own name, so a backup is still one
//! zip holding one database file.

mod controller;
mod restore;
mod service;

pub use controller::routes;
pub use restore::{extract_database, DatabaseRestoreService};
pub use service::{BackupFile, BackupReason, BackupStatus, DatabaseBackupService, DatabaseLocation};
