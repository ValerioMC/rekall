//! `ClaudeCodeInstaller` and `ClaudeCodeController`: registers Rekall's MCP endpoint with Claude
//! Code at user scope, clears the copies single folders carry, and installs the `/rk` command.

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use rekall_api::error::ApiResult;
use rekall_common::RekallError;
use serde::Serialize;
use serde_json::Value;
use tracing::{debug, info, warn};

pub const SERVER_NAME: &str = "rekall";

pub const CONNECTED: &str = "CONNECTED";
pub const OUTDATED: &str = "OUTDATED";
pub const NOT_CONNECTED: &str = "NOT_CONNECTED";
pub const CLI_MISSING: &str = "CLI_MISSING";

const COMMAND_FILE: &str = "rk.md";
/// The slash command this build ships: the repository's own `.claude/commands/rk.md`.
pub const PACKAGED_COMMAND: &str = include_str!("../../../.claude/commands/rk.md");
const HOME_RELATIVE_BINARIES: [&str; 3] = [".local/bin/claude", ".claude/local/claude", "bin/claude"];
const KNOWN_DIRECTORIES: &str = "/opt/homebrew/bin:/usr/local/bin:/usr/bin";
const COMMAND_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Installation {
    pub status: &'static str,
    pub endpoint: String,
    pub registered_url: Option<String>,
    pub folder_scoped: Vec<String>,
    pub command_installed: bool,
    pub cli_path: Option<String>,
    pub manual_command: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub exit_code: i32,
    pub output: String,
}

impl Outcome {
    pub fn succeeded(&self) -> bool {
        self.exit_code == 0
    }
}

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

pub fn routes(installer: ClaudeCodeInstaller) -> Router {
    Router::new()
        .route("/api/settings/claude", get(status))
        .route("/api/settings/claude/install", post(install))
        .with_state(installer)
}

async fn status(State(installer): State<ClaudeCodeInstaller>) -> Json<Installation> {
    let answer = tokio::task::spawn_blocking(move || installer.status()).await.expect("status never panics");
    Json(answer)
}

async fn install(State(installer): State<ClaudeCodeInstaller>) -> ApiResult<Json<Installation>> {
    let answer = tokio::task::spawn_blocking(move || installer.install()).await.expect("install never panics");
    Ok(Json(answer?))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::Mutex;

    use super::*;

    const ENDPOINT: &str = "http://localhost:47355/mcp";

    /// `~/.claude.json` as Claude Code writes it, and the commands a runner saw.
    #[derive(Default)]
    struct Fixture {
        user_scope_url: Option<String>,
        folder_scoped: Vec<PathBuf>,
        commands: Vec<Vec<String>>,
        directories: Vec<PathBuf>,
    }

    struct Harness {
        home: tempfile::TempDir,
        fixture: Arc<Mutex<Fixture>>,
    }

    impl Harness {
        fn new() -> Self {
            Self { home: tempfile::tempdir().unwrap(), fixture: Arc::new(Mutex::new(Fixture::default())) }
        }

        fn home(&self) -> &Path {
            self.home.path()
        }

        fn installer(&self, runner: CommandRunner) -> ClaudeCodeInstaller {
            ClaudeCodeInstaller::with_runner(self.home().to_path_buf(), ENDPOINT.into(), runner, Some(String::new()))
        }

        /// Records every command and answers success, writing nothing.
        fn recording(&self) -> CommandRunner {
            let fixture = self.fixture.clone();
            Arc::new(move |command: &[String], _: &HashMap<String, String>, directory: &Path| {
                let mut f = fixture.lock().unwrap();
                f.commands.push(command.to_vec());
                f.directories.push(directory.to_path_buf());
                Outcome { exit_code: 0, output: String::new() }
            })
        }

        /// Stands in for the real CLI: it registers or removes what it is asked to.
        fn realistic(&self) -> CommandRunner {
            let fixture = self.fixture.clone();
            let home = self.home().to_path_buf();
            Arc::new(move |command: &[String], _: &HashMap<String, String>, directory: &Path| {
                let mut f = fixture.lock().unwrap();
                f.commands.push(command.to_vec());
                f.directories.push(directory.to_path_buf());
                let has = |word: &str| command.iter().any(|c| c == word);
                if has("add") {
                    f.user_scope_url = command.last().cloned();
                } else if has("remove") && has("local") {
                    f.folder_scoped.retain(|folder| folder != directory);
                } else if has("remove") {
                    f.user_scope_url = None;
                }
                write_configuration(&home, &f);
                Outcome { exit_code: 0, output: String::new() }
            })
        }

        fn fake_cli(&self) -> PathBuf {
            let cli = self.home().join(".local/bin/claude");
            std::fs::create_dir_all(cli.parent().unwrap()).unwrap();
            std::fs::write(&cli, "#!/bin/sh\n").unwrap();
            std::fs::set_permissions(&cli, std::fs::Permissions::from_mode(0o755)).unwrap();
            cli
        }

        fn project_folder(&self, name: &str) -> PathBuf {
            let folder = self.home().join("Projects").join(name);
            std::fs::create_dir_all(&folder).unwrap();
            folder
        }

        fn register_user_scope(&self, url: &str) {
            let mut f = self.fixture.lock().unwrap();
            f.user_scope_url = Some(url.into());
            write_configuration(self.home(), &f);
        }

        fn register_folder_scope(&self, folder: &Path) {
            let mut f = self.fixture.lock().unwrap();
            f.folder_scoped.push(folder.to_path_buf());
            write_configuration(self.home(), &f);
        }

        fn write_command_file(&self, content: &str) {
            let file = self.home().join(".claude/commands/rk.md");
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, content).unwrap();
        }

        fn commands(&self) -> Vec<Vec<String>> {
            self.fixture.lock().unwrap().commands.clone()
        }
    }

    fn write_configuration(home: &Path, fixture: &Fixture) {
        let user = fixture
            .user_scope_url
            .as_ref()
            .map(|url| format!("\"mcpServers\": {{ \"rekall\": {{ \"type\": \"http\", \"url\": \"{url}\" }} }},\n  "))
            .unwrap_or_default();
        let projects: Vec<String> = fixture
            .folder_scoped
            .iter()
            .map(|folder| format!("\"{}\": {{ \"mcpServers\": {{ \"rekall\": {{ \"url\": \"{ENDPOINT}\" }} }} }}", folder.display()))
            .collect();
        std::fs::write(
            home.join(".claude.json"),
            format!("{{\n  {user}\"projects\": {{\n    {}\n  }}\n}}", projects.join(",\n    ")),
        )
        .unwrap();
    }

    fn line(cli: &Path, rest: &[&str]) -> Vec<String> {
        args(&cli.to_string_lossy(), rest)
    }

    #[test]
    fn with_no_cli_and_nothing_registered_it_reports_what_to_run_by_hand() {
        let h = Harness::new();
        let status = h.installer(h.recording()).status();
        assert_eq!(status.status, CLI_MISSING);
        assert_eq!(status.registered_url, None);
        assert_eq!(status.cli_path, None);
        assert!(!status.command_installed);
        assert_eq!(status.manual_command, format!("claude mcp add --scope user --transport http rekall {ENDPOINT}"));
    }

    #[test]
    fn a_cli_on_this_machine_and_no_registration_is_not_connected_not_a_missing_cli() {
        let h = Harness::new();
        let cli = h.fake_cli();
        let status = h.installer(h.recording()).status();
        assert_eq!(status.status, NOT_CONNECTED);
        assert_eq!(status.cli_path, Some(cli.to_string_lossy().into_owned()));
    }

    #[test]
    fn the_user_scope_registration_and_a_current_command_file_together_are_connected() {
        let h = Harness::new();
        h.fake_cli();
        h.register_user_scope(ENDPOINT);
        h.write_command_file(PACKAGED_COMMAND);
        assert_eq!(h.installer(h.recording()).status().status, CONNECTED);
    }

    #[test]
    fn a_registration_pointing_at_another_port_is_outdated() {
        let h = Harness::new();
        h.fake_cli();
        h.register_user_scope("http://localhost:8080/mcp");
        h.write_command_file(PACKAGED_COMMAND);
        let status = h.installer(h.recording()).status();
        assert_eq!(status.status, OUTDATED);
        assert_eq!(status.registered_url.as_deref(), Some("http://localhost:8080/mcp"));
    }

    #[test]
    fn an_older_copy_of_the_slash_command_is_outdated_even_when_the_server_is_right() {
        let h = Harness::new();
        h.fake_cli();
        h.register_user_scope(ENDPOINT);
        h.write_command_file("an earlier version of the command");
        let status = h.installer(h.recording()).status();
        assert_eq!(status.status, OUTDATED);
        assert!(!status.command_installed);
    }

    #[test]
    fn registrations_made_per_folder_are_reported_and_only_for_the_folders_that_carry_one() {
        let h = Harness::new();
        h.fake_cli();
        let carrying = h.project_folder("carrying");
        h.project_folder("plain");
        h.register_folder_scope(&carrying);
        let status = h.installer(h.recording()).status();
        assert_eq!(status.folder_scoped, vec![carrying.to_string_lossy().into_owned()]);
        assert_eq!(status.registered_url, None);
        assert_eq!(status.status, NOT_CONNECTED);
    }

    #[test]
    fn a_folder_carrying_a_copy_of_its_own_is_not_connected_right_as_the_user_scope_one_is() {
        let h = Harness::new();
        h.fake_cli();
        h.register_user_scope(ENDPOINT);
        h.write_command_file(PACKAGED_COMMAND);
        h.register_folder_scope(&h.project_folder("carrying"));
        assert_eq!(h.installer(h.recording()).status().status, OUTDATED);
    }

    #[test]
    fn a_copy_left_behind_by_a_folder_that_is_gone_is_ignored_not_reported_forever() {
        let h = Harness::new();
        h.fake_cli();
        h.write_command_file(PACKAGED_COMMAND);
        {
            let mut f = h.fixture.lock().unwrap();
            f.user_scope_url = Some(ENDPOINT.into());
            f.folder_scoped.push(PathBuf::from("/Users/someone/Projects/deleted-last-year"));
            write_configuration(h.home(), &f);
        }
        let status = h.installer(h.recording()).status();
        assert!(status.folder_scoped.is_empty());
        assert_eq!(status.status, CONNECTED);
    }

    #[test]
    fn installing_removes_the_old_registration_adds_it_at_user_scope_writes_the_command() {
        let h = Harness::new();
        let cli = h.fake_cli();
        h.register_user_scope("http://localhost:8080/mcp");
        let status = h.installer(h.realistic()).install().unwrap();
        assert_eq!(
            h.commands(),
            vec![
                line(&cli, &["mcp", "remove", "--scope", "user", "rekall"]),
                line(&cli, &["mcp", "add", "--scope", "user", "--transport", "http", "rekall", ENDPOINT]),
            ]
        );
        assert_eq!(std::fs::read_to_string(h.home().join(".claude/commands/rk.md")).unwrap(), PACKAGED_COMMAND);
        assert_eq!(status.status, CONNECTED);
        assert_eq!(status.registered_url.as_deref(), Some(ENDPOINT));
    }

    #[test]
    fn installing_clears_the_copy_a_folder_carried_from_inside_that_folder() {
        let h = Harness::new();
        let cli = h.fake_cli();
        let carrying = h.project_folder("carrying");
        h.register_folder_scope(&carrying);
        let status = h.installer(h.realistic()).install().unwrap();
        assert!(h.commands().contains(&line(&cli, &["mcp", "remove", "--scope", "local", "rekall"])));
        assert_eq!(h.fixture.lock().unwrap().directories.last(), Some(&carrying));
        assert!(status.folder_scoped.is_empty());
        assert_eq!(status.status, CONNECTED);
    }

    #[test]
    fn a_folder_that_refuses_to_give_up_its_copy_does_not_fail_the_install() {
        let h = Harness::new();
        h.fake_cli();
        h.register_folder_scope(&h.project_folder("carrying"));
        let fixture = h.fixture.clone();
        let home = h.home().to_path_buf();
        let runner: CommandRunner = Arc::new(move |command: &[String], _: &HashMap<String, String>, _: &Path| {
            let mut f = fixture.lock().unwrap();
            f.commands.push(command.to_vec());
            if command.iter().any(|c| c == "local") {
                return Outcome { exit_code: 1, output: "Error: no such server".into() };
            }
            if command.iter().any(|c| c == "add") {
                f.user_scope_url = command.last().cloned();
                write_configuration(&home, &f);
            }
            Outcome { exit_code: 0, output: String::new() }
        });
        let status = h.installer(runner).install().unwrap();
        assert_eq!(status.registered_url.as_deref(), Some(ENDPOINT));
        assert_eq!(std::fs::read_to_string(h.home().join(".claude/commands/rk.md")).unwrap(), PACKAGED_COMMAND);
    }

    #[test]
    fn installing_over_a_correct_registration_is_a_no_op_that_still_reports_connected() {
        let h = Harness::new();
        h.fake_cli();
        h.register_user_scope(ENDPOINT);
        h.write_command_file(PACKAGED_COMMAND);
        assert_eq!(h.installer(h.recording()).install().unwrap().status, CONNECTED);
    }

    #[test]
    fn without_a_cli_installing_refuses_and_hands_over_the_command_instead() {
        let h = Harness::new();
        let refused = h.installer(h.recording()).install().unwrap_err();
        assert!(matches!(refused, RekallError::Conflict(_)));
        assert!(refused.message().contains(&format!("claude mcp add --scope user --transport http rekall {ENDPOINT}")));
        assert!(h.commands().is_empty());
    }

    #[test]
    fn a_cli_that_refuses_the_registration_surfaces_its_own_words_and_writes_nothing() {
        let h = Harness::new();
        h.fake_cli();
        let runner: CommandRunner = Arc::new(|command: &[String], _: &HashMap<String, String>, _: &Path| {
            if command.iter().any(|c| c == "add") {
                Outcome { exit_code: 1, output: "Error: invalid transport".into() }
            } else {
                Outcome { exit_code: 0, output: String::new() }
            }
        });
        let refused = h.installer(runner).install().unwrap_err();
        assert!(matches!(refused, RekallError::Conflict(_)));
        assert!(refused.message().contains("invalid transport"));
        assert!(!h.home().join(".claude/commands/rk.md").exists());
    }

    #[test]
    fn the_real_runner_merges_output_and_reports_the_exit_code() {
        let outcome = run_process(
            &["/bin/sh".into(), "-c".into(), "echo out; echo err >&2; exit 4".into()],
            &HashMap::new(),
            Path::new("/"),
        );
        assert_eq!(outcome.exit_code, 4);
        let lines: BTreeSet<&str> = outcome.output.lines().collect();
        assert_eq!(lines, BTreeSet::from(["out", "err"]));
    }
}
