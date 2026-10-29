//! The backups of the open database and putting one back, as the Settings panel uses them. Both
//! are ports of `rekall_core::backup`.

mod backup_service;

pub use backup_service::BackupService;
