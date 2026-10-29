//! Everything with a process, a file, a socket or an outbound HTTP call behind it. For Claude Code:
//! the `claude` TUI in a pseudo-terminal per task, reached through `PtyTerminals`; the
//! account's usage read for the meter; the worker that runs the queue one terminal at a time; the
//! transcripts read for the tokens each task and step cost; Claude Code's own home folder the
//! console edits. Around the catalog: which database to open (`bootstrap`), backups and restores,
//! restarting onto another database, the release check, the OS credential store, the probes that
//! check a stored credential, the Jira search and the advisory database. `rekall-app` wires it.

mod claude_router;
mod claude_state;
pub mod backup;
pub mod bootstrap;
pub mod cli;
pub mod config;
pub mod connection;
pub mod credentials;
pub mod installer;
pub mod jira;
pub mod login_shell;
pub mod probes;
pub mod pty;
pub mod queue;
pub mod resources;
pub mod restart;
pub mod secret;
pub mod transcript;
pub mod usage;
pub mod version;
pub mod vulnerability;

pub use claude_router::router;
pub use claude_state::ClaudeState;
pub use config::ClaudeConfig;
