use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::login_shell::is_executable;

use super::EnvironmentSource;

const HOME_RELATIVE: [&str; 3] = [".local/bin/claude", ".claude/local/claude", "bin/claude"];
const KNOWN_DIRECTORIES: [&str; 3] = ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"];

/// Set by a running Claude Code session; inherited, they make the new `claude` refuse to nest.
const SESSION_MARKERS: [&str; 2] = ["CLAUDECODE", "CLAUDE_CODE_ENTRYPOINT"];

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
#[path = "claude_cli_tests.rs"]
mod tests;
