//! Claude Code hosted in Rekall: the interactive `claude` TUI in a pseudo-terminal per task,
//! piped to the console over one WebSocket; the account's usage read for the meter; the run queue
//! that works through tasks one terminal at a time; the transcripts read for the tokens each
//! task and step cost; and the files of Claude Code's own home folder (instructions, skills,
//! memories) the console edits. Nothing about a terminal is persisted.

mod claude_router;
mod claude_state;
pub mod cli;
pub mod config;
pub mod credentials;
pub mod installer;
pub mod login_shell;
pub mod pty;
pub mod queue;
pub mod resources;
pub mod socket;
pub mod terminal;
pub mod transcript;
pub mod usage;

pub use claude_router::router;
pub use claude_state::ClaudeState;
pub use config::ClaudeConfig;
