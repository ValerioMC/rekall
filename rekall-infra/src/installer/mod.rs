//! `ClaudeCodeInstaller`: registers Rekall's MCP endpoint with Claude
//! Code at user scope, clears the copies single folders carry, and installs the `/rk` command.

mod claude_code_installer;
mod outcome;

pub use claude_code_installer::{SERVER_NAME, CONNECTED, CommandRunner, ClaudeCodeInstaller};
pub use outcome::Outcome;

use rekall_core::claude::Installation;
