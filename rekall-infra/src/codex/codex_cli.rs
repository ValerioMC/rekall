use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rekall_kernel::{RekallError, Result};
use tokio::process::Command;

use crate::cli::{search_path, EnvironmentSource};
use crate::login_shell::is_executable;

#[derive(Clone)]
pub struct CodexCli {
    home: PathBuf,
    shell: Arc<dyn EnvironmentSource>,
}

impl CodexCli {
    pub fn new(home: PathBuf, shell: Arc<dyn EnvironmentSource>) -> Self {
        Self { home, shell }
    }

    pub async fn locate(&self) -> Option<PathBuf> {
        let environment = self.environment().await;
        let search = search_path(environment.get("PATH").map(String::as_str));
        [self.home.join(".local/bin"), self.home.join("bin")]
            .into_iter()
            .chain(search.into_iter().map(PathBuf::from))
            .find_map(|folder| codex_in(&folder))
    }

    pub async fn require(&self) -> Result<PathBuf> {
        self.locate().await.ok_or_else(|| RekallError::conflict("Codex CLI was not found. Install Codex and try again."))
    }

    pub async fn environment(&self) -> HashMap<String, String> {
        let mut environment = self.shell.current().await;
        environment.entry("HOME".into()).or_insert_with(|| self.home.to_string_lossy().into_owned());
        environment
    }

    pub(super) async fn codex_home(&self) -> PathBuf {
        self.environment().await.get("CODEX_HOME").filter(|path| !path.is_empty()).map(PathBuf::from)
            .unwrap_or_else(|| self.home.join(".codex"))
    }

    pub(super) async fn command(&self, binary: &Path) -> Command {
        let mut command = Command::new(binary);
        command.current_dir(&self.home).env_clear().envs(self.environment().await).kill_on_drop(true);
        command
    }
}

fn codex_in(folder: &Path) -> Option<PathBuf> {
    let path = folder.join(if cfg!(windows) { "codex.exe" } else { "codex" });
    is_executable(&path).then_some(path)
}
