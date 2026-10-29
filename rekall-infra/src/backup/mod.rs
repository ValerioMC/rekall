//! Rotating copies of the open database in a `backups` folder beside it, and putting one back: the
//! `DatabaseBackups` and `DatabaseRestore` ports of `rekall_core::backup`.
//!
//! A backup is a consistent copy of the live database written with `VACUUM INTO`, zipped under
//! the database file's own name: one zip holding one database file.

mod database_backup_service;
mod database_location;
mod database_restore_service;

pub use database_backup_service::DatabaseBackupService;
pub use database_location::DatabaseLocation;
pub use database_restore_service::{extract_database, DatabaseRestoreService};
