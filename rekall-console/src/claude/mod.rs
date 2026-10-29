//! Claude Code as the console sees it: the account's usage on the meter, the files of its home
//! folder the Claude screen edits, and whether it knows Rekall. Each is reached through a port of
//! `rekall_core::claude`.

mod command;
mod service;

pub use service::claude_folder_service::ClaudeFolderService;
pub use service::claude_install_service::ClaudeInstallService;
pub use command::save_resource_command::SaveResourceCommand;
pub use service::usage_meter_service::UsageMeterService;
