use std::path::{Path, PathBuf};
use std::time::Duration;

use async_trait::async_trait;
use rekall_core::commit::{run_git, GitResult};
use rekall_core::worktree::{parse_listing, Worktree, WorktreeHost, WorktreeLayout, WorktreeStatus};
use rekall_kernel::{RekallError, Result};

const ADD_TIMEOUT: Duration = Duration::from_secs(120);
const REMOVE_TIMEOUT: Duration = Duration::from_secs(60);
const READ_TIMEOUT: Duration = Duration::from_secs(10);

/// Runs `git worktree` for the project's repository. It never touches a folder outside the
/// layout's root: every path it creates or removes is checked against it first.
#[derive(Clone, Debug)]
pub struct GitWorktreeHost {
    layout: WorktreeLayout,
}

impl GitWorktreeHost {
    pub fn new(layout: WorktreeLayout) -> Self {
        Self { layout }
    }

    async fn git(&self, folder: &Path, arguments: &[&str], timeout: Duration) -> Result<GitResult> {
        let answer = run_git(folder, arguments, None, timeout).await?;
        if !answer.ok() {
            return Err(RekallError::illegal(format!("git {} failed in {}: {}", arguments.join(" "), folder.display(), answer.error())));
        }
        Ok(answer)
    }

    fn lexical_folder_of(&self, listed: &Path) -> Option<PathBuf> {
        let root = std::fs::canonicalize(&self.layout.root).ok()?;
        let relative = listed.strip_prefix(&root).ok().or_else(|| listed.strip_prefix(&self.layout.root).ok())?;
        Some(self.layout.root.join(relative))
    }
}

#[async_trait]
impl WorktreeHost for GitWorktreeHost {
    async fn create(&self, project_folder: &Path, branch_name: &str, folder: &Path) -> Result<Worktree> {
        self.layout.require_branch(branch_name)?;
        self.layout.require_folder(folder)?;
        if let Some(parent) = folder.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| RekallError::illegal(format!("Could not make {}: {e}", parent.display())))?;
        }
        let target = folder.to_string_lossy();
        self.git(project_folder, &["worktree", "add", "-b", branch_name, &target, "HEAD"], ADD_TIMEOUT).await?;
        Ok(Worktree { project_folder: project_folder.to_path_buf(), folder: folder.to_path_buf(), branch: branch_name.to_string() })
    }

    async fn remove(&self, worktree: &Worktree) -> Result<()> {
        self.layout.require_folder(&worktree.folder)?;
        let target = worktree.folder.to_string_lossy();
        self.git(&worktree.project_folder, &["worktree", "remove", "--force", &target], REMOVE_TIMEOUT).await?;
        Ok(())
    }

    async fn list(&self, project_folder: &Path) -> Result<Vec<Worktree>> {
        let answer = self.git(project_folder, &["worktree", "list", "--porcelain"], READ_TIMEOUT).await?;
        Ok(parse_listing(project_folder, &answer.stdout)
            .into_iter()
            .filter_map(|listed| {
                let folder = self.lexical_folder_of(&listed.folder)?;
                Some(Worktree { folder, ..listed })
            })
            .collect())
    }

    async fn status(&self, worktree: &Worktree) -> Result<WorktreeStatus> {
        let changes = self.git(&worktree.folder, &["status", "--porcelain"], READ_TIMEOUT).await?;
        let base = self.git(&worktree.project_folder, &["rev-parse", "HEAD"], READ_TIMEOUT).await?;
        let range = format!("{}..HEAD", base.output());
        let ahead = self.git(&worktree.folder, &["rev-list", "--count", &range], READ_TIMEOUT).await?;
        let commits_ahead = ahead
            .output()
            .parse()
            .map_err(|_| RekallError::illegal(format!("git did not answer a number of commits: '{}'.", ahead.output())))?;
        Ok(WorktreeStatus { has_unsaved_changes: !changes.output().is_empty(), commits_ahead })
    }
}
