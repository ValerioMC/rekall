use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use rekall_common::RekallError;
use serde_json::Value;
use tracing::{debug, info, warn};

use super::{Installation, Outcome};

pub const SERVER_NAME: &str = "rekall";

pub const CONNECTED: &str = "CONNECTED";
pub const OUTDATED: &str = "OUTDATED";
pub const NOT_CONNECTED: &str = "NOT_CONNECTED";
pub const CLI_MISSING: &str = "CLI_MISSING";

const COMMAND_FILE: &str = "rk.md";
/// The slash command this build ships: the repository's own `.claude/commands/rk.md`.
pub const PACKAGED_COMMAND: &str = include_str!("../../../../.claude/commands/rk.md");
const HOME_RELATIVE_BINARIES: [&str; 3] = [".local/bin/claude", ".claude/local/claude", "bin/claude"];
const KNOWN_DIRECTORIES: &str = "/opt/homebrew/bin:/usr/local/bin:/usr/bin";
const COMMAND_TIMEOUT: Duration = Duration::from_secs(20);

/// Runs a command in a directory with extra environment; the real one starts a process.
pub type CommandRunner = Arc<dyn Fn(&[String], &HashMap<String, String>, &Path) -> Outcome + Send + Sync>;

#[derive(Clone)]
pub struct ClaudeCodeInstaller {
    home: PathBuf,
    endpoint: String,
    runner: CommandRunner,
    search_path: Option<String>,
}

impl ClaudeCodeInstaller {
    pub fn new(home: PathBuf, endpoint: String) -> Self {
        Self::with_runner(home, endpoint, Arc::new(run_process), Some(default_search_path()))
    }

    pub fn with_runner(home: PathBuf, endpoint: String, runner: CommandRunner, search_path: Option<String>) -> Self {
        Self { home, endpoint, runner, search_path }
    }

    pub fn status(&self) -> Installation {
        let registered = self.registered_url();
        let cli = self.locate_cli();
        let command = self.command_is_current();
        let folders = self.folder_scoped();
        let status = if registered.as_deref() == Some(self.endpoint.as_str()) && command && folders.is_empty() {
            CONNECTED
        } else if cli.is_none() {
            CLI_MISSING
        } else if registered.is_none() {
            NOT_CONNECTED
        } else {
            OUTDATED
        };
        Installation {
            status,
            endpoint: self.endpoint.clone(),
            registered_url: registered,
            folder_scoped: folders,
            command_installed: command,
            cli_path: cli,
            manual_command: self.manual_command(),
        }
    }

    pub fn install(&self) -> Result<Installation, RekallError> {
        let Some(cli) = self.locate_cli() else {
            return Err(RekallError::conflict(format!(
                "Claude Code's command line tool isn't on this machine, so the registration can't be written for you. \
                 Run this instead: {}",
                self.manual_command()
            )));
        };
        let environment = HashMap::from([("HOME".to_string(), self.home.to_string_lossy().into_owned())]);

        let removed = (self.runner)(&args(&cli, &["mcp", "remove", "--scope", "user", SERVER_NAME]), &environment, &self.home);
        if !removed.succeeded() {
            debug!("Nothing to remove before registering {SERVER_NAME}: {}", removed.output);
        }

        let added = (self.runner)(
            &args(&cli, &["mcp", "add", "--scope", "user", "--transport", "http", SERVER_NAME, &self.endpoint]),
            &environment,
            &self.home,
        );
        if !added.succeeded() {
            return Err(RekallError::conflict(format!("Claude Code refused the registration: {}", added.output)));
        }

        self.clear_folder_scoped(&cli, &environment);
        self.install_command()?;
        info!("Registered {SERVER_NAME} at user scope on {} and installed the /rk command", self.endpoint);
        Ok(self.status())
    }

    fn clear_folder_scoped(&self, cli: &str, environment: &HashMap<String, String>) {
        for folder in self.folder_scoped() {
            let outcome = (self.runner)(&args(cli, &["mcp", "remove", "--scope", "local", SERVER_NAME]), environment, Path::new(&folder));
            if outcome.succeeded() {
                info!("Removed the {SERVER_NAME} registration {folder} carried of its own");
            } else {
                warn!("Could not remove the {SERVER_NAME} registration in {folder}: {}", outcome.output);
            }
        }
    }

    fn registered_url(&self) -> Option<String> {
        self.configuration()
            .pointer(&format!("/mcpServers/{SERVER_NAME}/url"))
            .and_then(Value::as_str)
            .map(str::to_string)
    }

    fn folder_scoped(&self) -> Vec<String> {
        let configuration = self.configuration();
        let Some(projects) = configuration.get("projects").and_then(Value::as_object) else {
            return Vec::new();
        };
        projects
            .iter()
            .filter(|(folder, entry)| {
                entry.pointer(&format!("/mcpServers/{SERVER_NAME}")).is_some_and(Value::is_object)
                    && Path::new(folder.as_str()).is_dir()
            })
            .map(|(folder, _)| folder.clone())
            .collect()
    }

    fn configuration(&self) -> Value {
        let file = self.home.join(".claude.json");
        if !file.is_file() {
            return Value::Object(Default::default());
        }
        match std::fs::read(&file).map_err(|e| e.to_string()).and_then(|b| serde_json::from_slice(&b).map_err(|e| e.to_string())) {
            Ok(value) => value,
            Err(failed) => {
                warn!("Could not read {}, reporting Rekall as unregistered: {failed}", file.display());
                Value::Object(Default::default())
            }
        }
    }

    fn command_file(&self) -> PathBuf {
        self.home.join(".claude").join("commands").join(COMMAND_FILE)
    }

    fn command_is_current(&self) -> bool {
        let file = self.command_file();
        if !file.is_file() {
            return false;
        }
        match std::fs::read_to_string(&file) {
            Ok(text) => text == PACKAGED_COMMAND,
            Err(failed) => {
                warn!("Could not read {}: {failed}", file.display());
                false
            }
        }
    }

    fn install_command(&self) -> Result<(), RekallError> {
        let file = self.command_file();
        let failed = || RekallError::internal("UncheckedIOException", format!("Could not write {}", file.display()));
        std::fs::create_dir_all(file.parent().expect("has a parent")).map_err(|_| failed())?;
        std::fs::write(&file, PACKAGED_COMMAND).map_err(|_| failed())
    }

    fn locate_cli(&self) -> Option<String> {
        for candidate in HOME_RELATIVE_BINARIES {
            let path = self.home.join(candidate);
            if rekall_claude::login_shell::is_executable(&path) {
                return Some(path.to_string_lossy().into_owned());
            }
        }
        let search_path = self.search_path.as_deref()?;
        for directory in search_path.split(':') {
            if directory.trim().is_empty() {
                continue;
            }
            let candidate = Path::new(directory).join("claude");
            if rekall_claude::login_shell::is_executable(&candidate) {
                return Some(candidate.to_string_lossy().into_owned());
            }
        }
        None
    }

    fn manual_command(&self) -> String {
        format!("claude mcp add --scope user --transport http {SERVER_NAME} {}", self.endpoint)
    }
}

fn args(cli: &str, rest: &[&str]) -> Vec<String> {
    std::iter::once(cli.to_string()).chain(rest.iter().map(|s| s.to_string())).collect()
}

fn default_search_path() -> String {
    match std::env::var("PATH") {
        Ok(path) if !path.trim().is_empty() => format!("{path}:{KNOWN_DIRECTORIES}"),
        _ => KNOWN_DIRECTORIES.to_string(),
    }
}

/// Output is stdout then stderr, trimmed; a command past the timeout is killed.
fn run_process(command: &[String], environment: &HashMap<String, String>, directory: &Path) -> Outcome {
    let Some((program, rest)) = command.split_first() else {
        return Outcome { exit_code: -1, output: "empty command".into() };
    };
    let mut child = match Command::new(program)
        .args(rest)
        .envs(environment)
        .current_dir(directory)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(failed) => return Outcome { exit_code: -1, output: failed.to_string() },
    };
    let mut stdout = child.stdout.take().expect("piped");
    let mut stderr = child.stderr.take().expect("piped");
    let out = std::thread::spawn(move || {
        let mut text = String::new();
        let _ = stdout.read_to_string(&mut text);
        text
    });
    let err = std::thread::spawn(move || {
        let mut text = String::new();
        let _ = stderr.read_to_string(&mut text);
        text
    });
    let deadline = Instant::now() + COMMAND_TIMEOUT;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
        }
    };
    let output = format!("{}{}", out.join().unwrap_or_default(), err.join().unwrap_or_default()).trim().to_string();
    match status {
        Some(status) => Outcome { exit_code: status.code().unwrap_or(-1), output },
        None => Outcome {
            exit_code: -1,
            output: format!("`{}` did not finish in {} seconds", command.join(" "), COMMAND_TIMEOUT.as_secs()),
        },
    }
}

#[cfg(test)]
#[path = "../../../test/bootstrap/installer/claude_code_installer_tests.rs"]
mod tests;
