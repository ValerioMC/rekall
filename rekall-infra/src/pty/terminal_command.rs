//! The command a terminal runs in the person’s login shell environment.

use std::collections::HashMap;
use std::path::Path;

use portable_pty::CommandBuilder;

use crate::session::SessionOptions;

/// `binary` with the options and the `/rk` line, in `directory`, with nothing of Rekall's own
/// environment: only `environment`, which is the person's shell plus what a TUI needs.
pub(super) fn terminal_command(
    binary: &Path,
    options: &SessionOptions,
    first_line: &str,
    mut environment: HashMap<String, String>,
    directory: &Path,
) -> CommandBuilder {
    let mut command = CommandBuilder::new(binary);
    command.args(options.arguments());
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
#[path = "../../tests/unit/pty/terminal_command_tests.rs"]
mod tests;
