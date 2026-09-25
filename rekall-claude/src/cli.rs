//! Finds the `claude` binary and the environment to run it with. The environment is the user's
//! login shell's, so a terminal opened here finds what their own terminal finds; the Claude Code
//! install locations are still tried first, and `rekall.claude.cli-path` overrides everything.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::login_shell::{is_executable, LoginShellEnvironment};

const HOME_RELATIVE: [&str; 3] = [".local/bin/claude", ".claude/local/claude", "bin/claude"];
const KNOWN_DIRECTORIES: [&str; 3] = ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"];

/// Set by a running Claude Code session; inherited, they make the new `claude` refuse to nest.
const SESSION_MARKERS: [&str; 2] = ["CLAUDECODE", "CLAUDE_CODE_ENTRYPOINT"];

/// Where a login shell's environment comes from: the real one, or what a test says.
#[async_trait::async_trait]
pub trait EnvironmentSource: Send + Sync {
    async fn current(&self) -> HashMap<String, String>;
}

#[async_trait::async_trait]
impl EnvironmentSource for Arc<LoginShellEnvironment> {
    async fn current(&self) -> HashMap<String, String> {
        LoginShellEnvironment::current(self).await
    }
}

#[derive(Clone)]
pub struct ClaudeCli {
    override_path: String,
    home: PathBuf,
    login_shell: Arc<dyn EnvironmentSource>,
}

impl ClaudeCli {
    pub fn new(override_path: Option<&str>, home: PathBuf, login_shell: Arc<dyn EnvironmentSource>) -> Self {
        Self { override_path: override_path.map(|p| p.trim().to_string()).unwrap_or_default(), home, login_shell }
    }

    pub async fn locate(&self) -> Option<PathBuf> {
        if !self.override_path.is_empty() {
            let path = PathBuf::from(&self.override_path);
            return is_executable(&path).then_some(path);
        }
        for candidate in HOME_RELATIVE {
            let path = self.home.join(candidate);
            if is_executable(&path) {
                return Some(path);
            }
        }
        let environment = self.login_shell.current().await;
        for directory in search_path(environment.get("PATH").map(String::as_str)) {
            let path = Path::new(&directory).join("claude");
            if is_executable(&path) {
                return Some(path);
            }
        }
        None
    }

    /// The login shell's environment with `HOME` set, a `PATH` that still reaches the CLI's usual
    /// directories, and no marker of an enclosing Claude Code session.
    pub async fn environment(&self) -> HashMap<String, String> {
        let mut environment = self.login_shell.current().await;
        for marker in SESSION_MARKERS {
            environment.remove(marker);
        }
        let home = self.home.to_string_lossy().into_owned();
        if !home.trim().is_empty() {
            environment.entry("HOME".into()).or_insert(home);
        }
        let path = search_path(environment.get("PATH").map(String::as_str)).join(separator());
        environment.insert("PATH".into(), path);
        environment
    }
}

fn separator() -> &'static str {
    if cfg!(windows) {
        ";"
    } else {
        ":"
    }
}

/// The directories of `path` in order, then any of the usual install directories it lacks.
fn search_path(path: Option<&str>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let declared = path.map(|p| p.split(separator()).map(str::to_string).collect::<Vec<_>>()).unwrap_or_default();
    for part in declared.into_iter().filter(|p| !p.trim().is_empty()).chain(KNOWN_DIRECTORIES.iter().map(|d| d.to_string())) {
        if !out.contains(&part) {
            out.push(part);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixed(HashMap<String, String>);

    #[async_trait::async_trait]
    impl EnvironmentSource for Fixed {
        async fn current(&self) -> HashMap<String, String> {
            self.0.clone()
        }
    }

    fn cli_with(override_path: &str, environment: &[(&str, &str)]) -> ClaudeCli {
        let map = environment.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        ClaudeCli::new(Some(override_path), PathBuf::from("/home/test-user"), Arc::new(Fixed(map)))
    }

    #[tokio::test]
    async fn the_login_shells_variables_reach_the_terminal_its_path_first() {
        let cli = cli_with("", &[("PATH", "/home/me/.cargo/bin:/usr/bin:/bin"), ("RUSTUP_HOME", "/home/me/.rustup"), ("SSH_AUTH_SOCK", "/tmp/agent.sock")]);
        let environment = cli.environment().await;
        assert_eq!(environment["RUSTUP_HOME"], "/home/me/.rustup");
        assert_eq!(environment["SSH_AUTH_SOCK"], "/tmp/agent.sock");
        assert_eq!(environment["PATH"], "/home/me/.cargo/bin:/usr/bin:/bin:/opt/homebrew/bin:/usr/local/bin");
    }

    #[tokio::test]
    async fn an_enclosing_claude_code_sessions_markers_are_not_passed_on() {
        let cli = cli_with("", &[("PATH", "/usr/bin"), ("CLAUDECODE", "1"), ("CLAUDE_CODE_ENTRYPOINT", "cli")]);
        let environment = cli.environment().await;
        assert!(!environment.contains_key("CLAUDECODE") && !environment.contains_key("CLAUDE_CODE_ENTRYPOINT"));
    }

    #[tokio::test]
    async fn with_no_path_at_all_the_usual_install_directories_stand_in() {
        let environment = cli_with("", &[]).environment().await;
        assert_eq!(environment["PATH"], "/opt/homebrew/bin:/usr/local/bin:/usr/bin");
        assert!(environment.contains_key("HOME"));
    }

    #[tokio::test]
    async fn an_override_that_is_executable_wins_one_that_is_not_finds_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join("claude");
        std::fs::write(&binary, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        assert_eq!(cli_with(&binary.to_string_lossy(), &[]).locate().await, Some(binary.clone()));
        assert_eq!(cli_with(&dir.path().join("missing").to_string_lossy(), &[]).locate().await, None);
    }
}
