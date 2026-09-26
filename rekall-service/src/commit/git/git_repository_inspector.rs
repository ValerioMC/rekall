use std::path::{Path, PathBuf};

use rekall_common::jstr;

use super::{RepositoryStatus, run};
use super::git_command::COMMAND_TIMEOUT;

/// Read-only questions about a folder: is it a repository, which branch is checked out, who
/// would git commit as (what `git config` resolves inside the folder).
#[derive(Clone, Copy, Debug, Default)]
pub struct GitRepositoryInspector;

impl GitRepositoryInspector {
    pub async fn is_repository(&self, folder: Option<&str>) -> bool {
        self.inspect(folder).await.repository
    }

    pub async fn inspect(&self, folder: Option<&str>) -> RepositoryStatus {
        let Some(folder) = folder.filter(|f| !jstr::is_blank(f)) else {
            return RepositoryStatus::unset();
        };
        let path = jstr::strip(folder);
        let repo_folder = PathBuf::from(path);
        if !repo_folder.is_dir() {
            return RepositoryStatus::missing(path);
        }
        let inside = match run(&repo_folder, &["rev-parse", "--is-inside-work-tree"], None, COMMAND_TIMEOUT).await {
            Ok(inside) => inside,
            Err(_) => return RepositoryStatus::not_a_repository(path),
        };
        if !inside.ok() || inside.output() != "true" {
            return RepositoryStatus::not_a_repository(path);
        }
        RepositoryStatus {
            folder: Some(path.to_string()),
            exists: true,
            repository: true,
            branch: answer(&repo_folder, &["symbolic-ref", "--short", "-q", "HEAD"]).await,
            user_name: answer(&repo_folder, &["config", "--get", "user.name"]).await,
            user_email: answer(&repo_folder, &["config", "--get", "user.email"]).await,
        }
    }
}

async fn answer(folder: &Path, arguments: &[&str]) -> Option<String> {
    match run(folder, arguments, None, COMMAND_TIMEOUT).await {
        Ok(result) if result.ok() && !jstr::is_blank(result.output()) => Some(result.output().to_string()),
        _ => None,
    }
}
