use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use rekall_core::claude::ClaudeCodeSetup;
use rekall_kernel::{cleanup, InternalFailure, RekallError};
use serde_json::Value;
use tracing::{debug, info, warn};

use super::{Installation, Outcome};

pub const SERVER_NAME: &str = "rekall";

pub const CONNECTED: &str = "CONNECTED";
pub const OUTDATED: &str = "OUTDATED";
pub const NOT_CONNECTED: &str = "NOT_CONNECTED";
pub const CLI_MISSING: &str = "CLI_MISSING";

const COMMAND_FILE: &str = "rk.md";
const MODES_DIRECTORY: &str = "rk";
/// The slash command this build ships: the repository's own `.claude/commands/rk.md`.
pub const PACKAGED_COMMAND: &str = include_str!("../../../.claude/commands/rk.md");
/// The mode files `rk.md` sends a session to, from the repository's `.claude/rk/`, as name and text.
pub const PACKAGED_MODES: [(&str, &str); 13] = [
    ("assessing.md", include_str!("../../../.claude/rk/assessing.md")),
    ("autoconfiguring.md", include_str!("../../../.claude/rk/autoconfiguring.md")),
    ("explaining.md", include_str!("../../../.claude/rk/explaining.md")),
    ("generating.md", include_str!("../../../.claude/rk/generating.md")),
    ("inventorying.md", include_str!("../../../.claude/rk/inventorying.md")),
    ("merging.md", include_str!("../../../.claude/rk/merging.md")),
    ("noting.md", include_str!("../../../.claude/rk/noting.md")),
    ("planning.md", include_str!("../../../.claude/rk/planning.md")),
    ("replanning.md", include_str!("../../../.claude/rk/replanning.md")),
    ("revising.md", include_str!("../../../.claude/rk/revising.md")),
    ("sectioning.md", include_str!("../../../.claude/rk/sectioning.md")),
    ("stepping.md", include_str!("../../../.claude/rk/stepping.md")),
    ("wrapping-up.md", include_str!("../../../.claude/rk/wrapping-up.md")),
];
const HOME_RELATIVE_DIRECTORIES: [&str; 3] = [".local/bin", ".claude/local", "bin"];
/// What the binary is called: the native installer writes `claude.exe` on Windows and npm a `claude.cmd` shim.
const BINARY_NAMES: &[&str] = if cfg!(windows) { &["claude.exe", "claude.cmd"] } else { &["claude"] };
const KNOWN_DIRECTORIES: [&str; 3] = ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"];
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
        let outdated_files = self.outdated_files();
        let folders = self.folder_scoped();
        let status = if registered.as_deref() == Some(self.endpoint.as_str()) && outdated_files.is_empty() && folders.is_empty() {
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
            command_installed: outdated_files.is_empty(),
            outdated_files,
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
        let home = self.home.to_string_lossy().into_owned();
        let environment = HashMap::from([("HOME".to_string(), home.clone()), ("USERPROFILE".to_string(), home)]);

        let removed = (self.runner)(&args(&cli, &["mcp", "remove", "--scope", "user", SERVER_NAME]), &environment, &self.home);
        if !removed.succeeded() {
            debug!(server = SERVER_NAME, output = %removed.output, "Nothing to remove before registering");
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
        info!(server = SERVER_NAME, endpoint = %self.endpoint, "Registered at user scope and installed the /rk command");
        Ok(self.status())
    }

    fn clear_folder_scoped(&self, cli: &str, environment: &HashMap<String, String>) {
        for folder in self.folder_scoped() {
            let outcome = (self.runner)(&args(cli, &["mcp", "remove", "--scope", "local", SERVER_NAME]), environment, Path::new(&folder));
            if outcome.succeeded() {
                info!(server = SERVER_NAME, %folder, "Removed a project-scoped registration");
            } else {
                warn!(server = SERVER_NAME, %folder, output = %outcome.output, "Could not remove a project-scoped registration");
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
                warn!(%failed, file = %file.display(), "Could not read the Claude Code settings; reporting Rekall as unregistered");
                Value::Object(Default::default())
            }
        }
    }

    fn packaged_files(&self) -> Vec<(PathBuf, &'static str)> {
        let modes = self.home.join(".claude").join(MODES_DIRECTORY);
        let command = (self.home.join(".claude").join("commands").join(COMMAND_FILE), PACKAGED_COMMAND);
        std::iter::once(command).chain(PACKAGED_MODES.iter().map(|(name, text)| (modes.join(name), *text))).collect()
    }

    fn outdated_files(&self) -> Vec<String> {
        let outdated: Vec<String> = self
            .packaged_files()
            .into_iter()
            .filter(|(file, text)| !self.file_holds(file, text))
            .map(|(file, _)| self.relative_to_claude_folder(&file))
            .collect();
        if !outdated.is_empty() {
            info!(files = %outdated.join(", "), "The installed /rk files differ from this build's");
        }
        outdated
    }

    fn file_holds(&self, file: &Path, text: &str) -> bool {
        match std::fs::read_to_string(file) {
            Ok(found) => found == text,
            Err(failed) if failed.kind() == std::io::ErrorKind::NotFound => false,
            Err(failed) => {
                warn!(%failed, file = %file.display(), "Could not read an installed /rk file");
                false
            }
        }
    }

    fn relative_to_claude_folder(&self, file: &Path) -> String {
        let folder = self.home.join(".claude");
        file.strip_prefix(&folder).unwrap_or(file).to_string_lossy().into_owned()
    }

    fn install_command(&self) -> Result<(), RekallError> {
        for (file, text) in self.packaged_files() {
            let failed = || RekallError::internal(InternalFailure::Io, format!("Could not write {}", file.display()));
            std::fs::create_dir_all(file.parent().expect("an installed file lies in a folder under the home")).map_err(|_| failed())?;
            std::fs::write(&file, text).map_err(|_| failed())?;
        }
        Ok(())
    }

    fn locate_cli(&self) -> Option<String> {
        let home_directories = HOME_RELATIVE_DIRECTORIES.iter().map(|directory| self.home.join(directory));
        let path_directories = self.search_path.iter().flat_map(|search_path| std::env::split_paths(search_path));
        home_directories
            .chain(path_directories)
            .filter(|directory| !directory.as_os_str().is_empty())
            .find_map(|directory| binary_in(&directory))
    }

    fn manual_command(&self) -> String {
        format!("claude mcp add --scope user --transport http {SERVER_NAME} {}", self.endpoint)
    }
}

fn args(cli: &str, rest: &[&str]) -> Vec<String> {
    std::iter::once(cli.to_string()).chain(rest.iter().map(|s| s.to_string())).collect()
}

fn binary_in(directory: &Path) -> Option<String> {
    BINARY_NAMES
        .iter()
        .map(|name| directory.join(name))
        .find(|candidate| crate::login_shell::is_executable(candidate))
        .map(|found| found.to_string_lossy().into_owned())
}

/// `PATH` followed by the directories a desktop app, started without a login shell, would miss.
fn default_search_path() -> String {
    let known = if cfg!(windows) { Vec::new() } else { KNOWN_DIRECTORIES.iter().map(PathBuf::from).collect() };
    let inherited = std::env::var_os("PATH").map(|path| std::env::split_paths(&path).collect::<Vec<_>>()).unwrap_or_default();
    std::env::join_paths(inherited.into_iter().chain(known))
        .map(|joined| joined.to_string_lossy().into_owned())
        .unwrap_or_default()
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
    let stdout = child.stdout.take().expect("both pipes were requested at spawn");
    let stderr = child.stderr.take().expect("both pipes were requested at spawn");
    let out = std::thread::spawn(move || drain(stdout));
    let err = std::thread::spawn(move || drain(stderr));
    let deadline = Instant::now() + COMMAND_TIMEOUT;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
            _ => {
                cleanup::kill(&mut child);
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

/// What the pipe held until it closed; a read that fails part-way keeps what it had and logs why.
impl ClaudeCodeSetup for ClaudeCodeInstaller {
    fn status(&self) -> Installation {
        ClaudeCodeInstaller::status(self)
    }

    fn install(&self) -> Result<Installation, RekallError> {
        ClaudeCodeInstaller::install(self)
    }
}

fn drain(mut pipe: impl Read) -> String {
    let mut text = String::new();
    if let Err(failed) = pipe.read_to_string(&mut text) {
        debug!(%failed, "Stopped reading a command's output");
    }
    text
}

#[cfg(test)]
#[path = "../../tests/unit/installer/claude_code_installer_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/unit/installer/packaged_command_tests.rs"]
mod packaged_command_tests;
