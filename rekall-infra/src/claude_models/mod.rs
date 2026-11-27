//! The model catalog of the installed Claude Code, read from the CLI itself so a new release shows
//! up with no change to Rekall.

mod cli_claude_model_reader;

pub use cli_claude_model_reader::CliClaudeModelReader;
