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
