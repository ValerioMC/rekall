//! The `claude` command a terminal runs: the flags a session starts with, and the environment of
//! the person's own shell around it.

use std::collections::HashMap;
use std::path::Path;

use portable_pty::CommandBuilder;

use crate::session::ClaudeOptions;

/// `binary` with the options and the `/rk` line, in `directory`, with nothing of Rekall's own
/// environment: only `environment`, which is the person's shell plus what a TUI needs.
pub(super) fn claude_command(
    binary: &Path,
    options: &ClaudeOptions,
    first_line: &str,
    mut environment: HashMap<String, String>,
    directory: &Path,
) -> CommandBuilder {
    let mut command = CommandBuilder::new(binary);
    if options.skip_permissions {
        command.arg("--dangerously-skip-permissions");
    }
    if let Some(model) = &options.model {
        command.args(["--model", model]);
    }
    if let Some(effort) = &options.effort {
        command.args(["--effort", effort]);
    }
    if let Some(settings) = &options.settings {
        command.arg("--settings");
        command.arg(settings);
    }
    command.arg(first_line);
    command.env_clear();
    environment.insert("TERM".into(), "xterm-256color".into());
    environment.insert("COLORTERM".into(), "truecolor".into());
    for (key, value) in environment {
        command.env(key, value);
    }
    command.cwd(directory);
    command
}

#[cfg(test)]
#[path = "../../tests/unit/pty/claude_command_tests.rs"]
mod tests;
