use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;

use rekall_core::claude::SessionProvider;
use tokio::process::Command;

use crate::session::SessionOptions;

const HEADLESS_ARGUMENTS: [&str; 7] =
    ["-p", "--verbose", "--input-format", "stream-json", "--output-format", "stream-json", "--include-partial-messages"];

pub fn chat_command(binary: &Path, options: &SessionOptions, environment: HashMap<String, String>, directory: &Path) -> Command {
    let mut command = Command::new(binary);
    match options.provider {
        SessionProvider::Claude => {
            command.args(HEADLESS_ARGUMENTS);
            if !options.skip_permissions {
                command.args(["--permission-prompt-tool", "stdio"]);
            }
            command.args(options.arguments());
        }
        SessionProvider::Codex => {
            command.arg("app-server");
        }
    }
    command.env_clear().envs(environment).current_dir(directory);
    command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true);
    command
}

#[cfg(test)]
#[path = "../../tests/unit/chat/chat_command_tests.rs"]
mod tests;
