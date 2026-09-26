//! Claude Code hosted in Rekall: the interactive `claude` TUI in a pseudo-terminal per task,
//! piped to the console over one WebSocket; the account's usage read for the meter; and the run
//! queue that works through tasks one terminal at a time. Nothing about a terminal is persisted.

mod claude_router;
mod claude_state;
pub mod cli;
pub mod config;
pub mod credentials;
pub mod login_shell;
pub mod pty;
pub mod queue;
pub mod socket;
pub mod terminal;
pub mod usage;

pub use claude_router::router;
pub use claude_state::ClaudeState;
pub use config::ClaudeConfig;
