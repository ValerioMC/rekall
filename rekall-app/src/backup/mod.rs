//! Rotating copies of the open database in a `backups` folder beside it, and putting one back.
//!
//! H2 wrote a backup with `BACKUP TO`; SQLite writes a consistent copy of a live database with
//! `VACUUM INTO`, which this zips under the database file's own name, so a backup is still one
//! zip holding one database file.

mod backup_controller;
mod backup_file;
mod backup_reason;
mod backup_state;
mod backup_status;
mod database_backup_service;
mod database_location;
mod database_restore_service;
mod restore_started;

pub use backup_controller::routes;
pub use backup_file::BackupFile;
pub use backup_reason::BackupReason;
pub use backup_state::BackupState;
pub use backup_status::BackupStatus;
pub use database_backup_service::DatabaseBackupService;
pub use database_location::DatabaseLocation;
pub use database_restore_service::{extract_database, DatabaseRestoreService};

use restore_started::RestoreStarted;
