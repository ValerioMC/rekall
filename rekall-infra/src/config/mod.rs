//! The leaf settings the infrastructure reads: the property sources, the database override, the
//! backup schedule, the update check and Claude Code's own settings. `AppConfig` in `rekall-app`
//! composes them.

mod backup_config;
mod claude_config;
mod database_override;
mod properties;
mod update_check_config;

pub use backup_config::BackupConfig;
pub use claude_config::{claude_home_under, transcripts_under, ClaudeConfig};
pub use database_override::DatabaseOverride;
pub use properties::Properties;
pub use update_check_config::UpdateCheckConfig;
