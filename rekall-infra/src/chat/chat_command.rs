//! The headless `claude` a chat session runs: stream-json on both pipes, and the environment of
//! the person's own shell around it.

use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;

use tokio::process::Command;

use crate::session::ClaudeOptions;

const HEADLESS_ARGUMENTS: [&str; 7] =
    ["-p", "--verbose", "--input-format", "stream-json", "--output-format", "stream-json", "--include-partial-messages"];

/// `binary` in `directory` with only `environment`. Without skip-permissions, `claude` asks its
/// host for each approval over the same pipes.
pub fn chat_command(binary: &Path, options: &ClaudeOptions, environment: HashMap<String, String>, directory: &Path) -> Command {
    let mut command = Command::new(binary);
    command.args(HEADLESS_ARGUMENTS);
    if options.skip_permissions {
        command.arg("--dangerously-skip-permissions");
    } else {
        command.args(["--permission-prompt-tool", "stdio"]);
    }
    if let Some(model) = &options.model {
        command.args(["--model", model]);
    }
    if let Some(effort) = &options.effort {
        command.args(["--effort", effort]);
    }
    if let Some(settings) = &options.settings {
        command.arg("--settings").arg(settings);
    }
    command.env_clear().envs(environment).current_dir(directory);
    command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true);
    command
}

#[cfg(test)]
#[path = "../../tests/unit/chat/chat_command_tests.rs"]
mod tests;
