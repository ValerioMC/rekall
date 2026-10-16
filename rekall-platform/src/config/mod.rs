//! The leaf settings the platform reads: the property sources, the database override, the backup
//! schedule and the update check. `AppConfig` in `rekall-app` composes them.

mod backup_config;
mod database_override;
mod properties;
mod update_check_config;

pub use backup_config::BackupConfig;
pub use database_override::DatabaseOverride;
pub use properties::Properties;
pub use update_check_config::UpdateCheckConfig;
