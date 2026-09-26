//! Finds the `claude` binary and the environment to run it with. The environment is the user's
//! login shell's, so a terminal opened here finds what their own terminal finds; the Claude Code
//! install locations are still tried first, and `rekall.claude.cli-path` overrides everything.

mod claude_cli;
mod environment_source;

pub use claude_cli::ClaudeCli;
pub use environment_source::EnvironmentSource;
