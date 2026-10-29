//! Claude Code as the console sees it: the account's usage on the meter, the files of its home
//! folder the Claude screen edits, and whether it knows Rekall. Each is reached through a port of
//! `rekall_core::claude`.

mod claude_folder_service;
mod claude_install_service;
mod save_resource_command;
mod usage_meter_service;

pub use claude_folder_service::ClaudeFolderService;
pub use claude_install_service::ClaudeInstallService;
pub use save_resource_command::SaveResourceCommand;
pub use usage_meter_service::UsageMeterService;
