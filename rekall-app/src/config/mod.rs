//! The application's properties: a `--name=value` command-line argument wins, then an environment
//! variable in relaxed form (`server.port` from `SERVER_PORT`, `rekall.claude.cli-path` from
//! `REKALL_CLAUDE_CLIPATH`), then the default declared in `AppConfig`.

mod app_config;
mod backup_config;
mod database_override;
mod properties;
mod update_check_config;

pub use app_config::{DEFAULT_PORT, AppConfig};
pub use backup_config::BackupConfig;
pub use database_override::DatabaseOverride;
pub use properties::Properties;
pub use update_check_config::UpdateCheckConfig;
