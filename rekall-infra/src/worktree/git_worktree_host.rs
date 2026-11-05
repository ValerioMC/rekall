use std::path::{Path, PathBuf};
use std::time::Duration;

use async_trait::async_trait;
use rekall_core::commit::{run_git, GitResult};
use rekall_core::worktree::{parse_listing, BranchReview, MergeOutcome, Worktree, WorktreeHost, WorktreeLayout, WorktreeStatus};
use rekall_kernel::{RekallError, Result};
use tracing::{debug, warn};

const ADD_TIMEOUT: Duration = Duration::from_secs(120);
const REMOVE_TIMEOUT: Duration = Duration::from_secs(60);
const MERGE_TIMEOUT: Duration = Duration::from_secs(120);
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

    /// `git merge-tree --write-tree` (git 2.38 and later) says whether a merge would conflict
    /// without touching the work tree, the index or any branch; older git answers none.
    async fn merges_cleanly(&self, project_folder: &Path, branch_name: &str) -> Option<bool> {
        let answer = run_git(project_folder, &["merge-tree", "--write-tree", "HEAD", branch_name], None, READ_TIMEOUT).await.ok()?;
        match answer.exit_code {
            0 => Some(true),
            1 => Some(false),
            _ => None,
        }
    }

    async fn abandon_merge(&self, project_folder: &Path) {
        match run_git(project_folder, &["merge", "--abort"], None, READ_TIMEOUT).await {
            Ok(answer) if !answer.ok() => debug!(reason = answer.error(), "No merge was left to abandon"),
            Ok(_) => {}
            Err(failure) => warn!(%failure, folder = %project_folder.display(), "Could not abandon a refused merge"),
        }
    }

    async fn branch_exists(&self, project_folder: &Path, branch_name: &str) -> Result<bool> {
        let reference = format!("refs/heads/{branch_name}");
        let answer = run_git(project_folder, &["rev-parse", "--verify", "--quiet", &reference], None, READ_TIMEOUT).await?;
        Ok(answer.ok())
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
        if self.branch_exists(project_folder, branch_name).await? {
            self.git(project_folder, &["worktree", "add", &target, branch_name], ADD_TIMEOUT).await?;
        } else {
            self.git(project_folder, &["worktree", "add", "-b", branch_name, &target, "HEAD"], ADD_TIMEOUT).await?;
        }
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

    async fn review_branch(&self, project_folder: &Path, branch_name: &str, worktree_folder: Option<&Path>) -> Result<Option<BranchReview>> {
        self.layout.require_branch(branch_name)?;
        if !self.branch_exists(project_folder, branch_name).await? {
            return Ok(None);
        }
        let range = format!("HEAD..{branch_name}");
        let ahead = self.git(project_folder, &["rev-list", "--count", &range], READ_TIMEOUT).await?;
        let commits_ahead = ahead.output().parse().map_err(|_| RekallError::illegal(format!("git did not answer a number of commits: '{}'.", ahead.output())))?;
        let since_fork = format!("HEAD...{branch_name}");
        let changed = self.git(project_folder, &["diff", "--name-only", &since_fork], READ_TIMEOUT).await?;
        let has_unsaved_changes = match worktree_folder {
            Some(folder) => !self.git(folder, &["status", "--porcelain"], READ_TIMEOUT).await?.output().is_empty(),
            None => false,
        };
        Ok(Some(BranchReview {
            branch: branch_name.to_string(),
            commits_ahead,
            changed_files: changed.output().lines().map(str::to_string).collect(),
            merges_cleanly: self.merges_cleanly(project_folder, branch_name).await,
            worktree_folder: worktree_folder.map(|folder| folder.to_string_lossy().into_owned()),
            has_unsaved_changes,
        }))
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

    async fn merge_branch(&self, project_folder: &Path, branch_name: &str) -> Result<MergeOutcome> {
        self.layout.require_branch(branch_name)?;
        let answer = run_git(project_folder, &["merge", "--no-edit", branch_name], None, MERGE_TIMEOUT).await?;
        if answer.ok() {
            return Ok(MergeOutcome::Merged);
        }
        self.abandon_merge(project_folder).await;
        let reason = [answer.error(), answer.output()].into_iter().find(|text| !text.is_empty()).unwrap_or("git refused the merge");
        Ok(MergeOutcome::Refused(reason.to_string()))
    }
}
