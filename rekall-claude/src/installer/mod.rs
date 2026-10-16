//! `ClaudeCodeInstaller` and `ClaudeCodeController`: registers Rekall's MCP endpoint with Claude
//! Code at user scope, clears the copies single folders carry, and installs the `/rk` command.

mod claude_code_installer;
mod installation;
mod installer_controller;
mod outcome;

pub use claude_code_installer::{SERVER_NAME, CONNECTED, CommandRunner, ClaudeCodeInstaller};
pub use installation::Installation;
pub use installer_controller::routes;
pub use outcome::Outcome;
