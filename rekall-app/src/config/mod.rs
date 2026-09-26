//! The application's properties, read the way Spring Boot read `application.yaml`'s: a
//! `--name=value` command-line argument wins, then an environment variable in Spring's relaxed
//! form (`server.port` from `SERVER_PORT`, `rekall.claude.cli-path` from `REKALL_CLAUDE_CLIPATH`),
//! then the default the Java code declared.

mod app_config;
mod backup_config;
mod database_override;
mod properties;

pub use app_config::{DEFAULT_PORT, AppConfig};
pub use backup_config::BackupConfig;
pub use database_override::DatabaseOverride;
pub use properties::Properties;
