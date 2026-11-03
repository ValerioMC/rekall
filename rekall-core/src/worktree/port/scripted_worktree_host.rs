use std::collections::HashSet;
use std::path::Path;
use std::sync::Mutex;

use async_trait::async_trait;
use rekall_kernel::{RekallError, Result};

use crate::worktree::{BranchReview, Worktree, WorktreeHost, WorktreeStatus};

/// A `WorktreeHost` that keeps its worktrees in memory and makes their folders on disk, for tests
/// of what decides when to make one. `refuse` makes `create` fail for a branch.
#[derive(Default)]
pub struct ScriptedWorktreeHost {
    worktrees: Mutex<Vec<Worktree>>,
    refused: Mutex<HashSet<String>>,
    dirty: Mutex<HashSet<String>>,
}

impl ScriptedWorktreeHost {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn refuse(&self, branch_name: &str) {
        self.refused.lock().expect("scripted host lock").insert(branch_name.to_string());
    }

    pub fn leave_unsaved_changes(&self, branch_name: &str) {
        self.dirty.lock().expect("scripted host lock").insert(branch_name.to_string());
    }

    pub fn branches(&self) -> Vec<String> {
        self.worktrees.lock().expect("scripted host lock").iter().map(|worktree| worktree.branch.clone()).collect()
    }
}

#[async_trait]
impl WorktreeHost for ScriptedWorktreeHost {
    async fn create(&self, project_folder: &Path, branch_name: &str, folder: &Path) -> Result<Worktree> {
        if self.refused.lock().expect("scripted host lock").contains(branch_name) {
            return Err(RekallError::illegal(format!("git could not make a worktree for {branch_name}.")));
        }
        std::fs::create_dir_all(folder).map_err(|e| RekallError::illegal(format!("Could not make {}: {e}", folder.display())))?;
        let worktree = Worktree { project_folder: project_folder.to_path_buf(), folder: folder.to_path_buf(), branch: branch_name.to_string() };
        self.worktrees.lock().expect("scripted host lock").push(worktree.clone());
        Ok(worktree)
    }

    async fn remove(&self, worktree: &Worktree) -> Result<()> {
        let _ = std::fs::remove_dir_all(&worktree.folder);
        self.worktrees.lock().expect("scripted host lock").retain(|kept| kept != worktree);
        Ok(())
    }

    async fn list(&self, project_folder: &Path) -> Result<Vec<Worktree>> {
        Ok(self.worktrees.lock().expect("scripted host lock").iter().filter(|worktree| worktree.project_folder == project_folder).cloned().collect())
    }

    async fn review_branch(&self, _: &Path, branch_name: &str, worktree_folder: Option<&Path>) -> Result<Option<BranchReview>> {
        let known = self.worktrees.lock().expect("scripted host lock").iter().any(|worktree| worktree.branch == branch_name);
        let dirty = self.dirty.lock().expect("scripted host lock").contains(branch_name);
        Ok(known.then(|| BranchReview {
            branch: branch_name.to_string(),
            commits_ahead: 0,
            changed_files: Vec::new(),
            merges_cleanly: Some(true),
            worktree_folder: worktree_folder.map(|folder| folder.to_string_lossy().into_owned()),
            has_unsaved_changes: dirty,
        }))
    }

    async fn status(&self, worktree: &Worktree) -> Result<WorktreeStatus> {
        let dirty = self.dirty.lock().expect("scripted host lock").contains(&worktree.branch);
        Ok(WorktreeStatus { has_unsaved_changes: dirty, commits_ahead: 0 })
    }
}
