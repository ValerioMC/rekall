//! The `claude` command a terminal runs: the flags a session starts with, and the environment of
//! the person's own shell around it.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use portable_pty::CommandBuilder;
use rekall_core::claude::{EFFORT_LEVELS, MODEL_ALIASES};
use rekall_kernel::jstr;

/// How a session is started, with the model and effort already narrowed to the offered values.
#[derive(Clone, Debug, Default)]
pub(super) struct ClaudeOptions {
    pub skip_permissions: bool,
    pub model: Option<String>,
    pub effort: Option<String>,
    /// A credential session's settings, whose deny rules guard its helpers.
    pub settings: Option<PathBuf>,
}

impl ClaudeOptions {
    pub(super) fn chosen(skip_permissions: bool, model: Option<&str>, effort: Option<&str>) -> Self {
        Self { skip_permissions, model: normalise(model, &MODEL_ALIASES), effort: normalise(effort, &EFFORT_LEVELS), settings: None }
    }
}

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

fn normalise(value: Option<&str>, allowed: &[&str]) -> Option<String> {
    let trimmed = jstr::strip(value?).to_lowercase();
    allowed.contains(&trimmed.as_str()).then_some(trimmed)
}

#[cfg(test)]
#[path = "../../tests/unit/pty/claude_command_tests.rs"]
mod tests;
