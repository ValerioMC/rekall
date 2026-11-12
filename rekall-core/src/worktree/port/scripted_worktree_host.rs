use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Mutex;

use async_trait::async_trait;
use rekall_kernel::{RekallError, Result};

use crate::worktree::{BranchReview, MergeOutcome, PushOutcome, Worktree, WorktreeHost, WorktreeStatus};

/// A `WorktreeHost` that keeps its worktrees in memory and makes their folders on disk, for tests
/// of what decides when to make one. `refuse` makes `create` fail for a branch; a branch can be
/// scripted to hold commits or to conflict, and `merged` lists the branches merged and `bases` the branch each started from or went into.
/// `refuse_push` makes pushes of a branch fail with a reason; a branch pushed twice is up to date the second time.
#[derive(Default)]
pub struct ScriptedWorktreeHost {
    worktrees: Mutex<Vec<Worktree>>,
    refused: Mutex<HashSet<String>>,
    dirty: Mutex<HashSet<String>>,
    ahead: Mutex<HashSet<String>>,
    conflicting: Mutex<HashSet<String>>,
    merged: Mutex<Vec<String>>,
    bases: Mutex<Vec<String>>,
    pushed: Mutex<Vec<String>>,
    push_refusals: Mutex<HashMap<String, String>>,
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

    pub fn leave_commits(&self, branch_name: &str) {
        self.ahead.lock().expect("scripted host lock").insert(branch_name.to_string());
    }

    pub fn conflict_on(&self, branch_name: &str) {
        self.conflicting.lock().expect("scripted host lock").insert(branch_name.to_string());
    }

    pub fn resolve_conflict(&self, branch_name: &str) {
        self.conflicting.lock().expect("scripted host lock").remove(branch_name);
    }

    pub fn refuse_push(&self, branch_name: &str, reason: &str) {
        self.push_refusals.lock().expect("scripted host lock").insert(branch_name.to_string(), reason.to_string());
    }

    /// The branches pushed, once each, in the order they first went.
    pub fn pushed(&self) -> Vec<String> {
        self.pushed.lock().expect("scripted host lock").clone()
    }

    pub fn merged(&self) -> Vec<String> {
        self.merged.lock().expect("scripted host lock").clone()
    }

    /// The base each worktree was started from and each merge went into, in the order they happened.
    pub fn bases(&self) -> Vec<String> {
        self.bases.lock().expect("scripted host lock").clone()
    }

    pub fn branches(&self) -> Vec<String> {
        self.worktrees.lock().expect("scripted host lock").iter().map(|worktree| worktree.branch.clone()).collect()
    }
}

#[async_trait]
impl WorktreeHost for ScriptedWorktreeHost {
    async fn create(&self, project_folder: &Path, branch_name: &str, folder: &Path, base: &str) -> Result<Worktree> {
        self.bases.lock().expect("scripted host lock").push(base.to_string());
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

    async fn review_branch(&self, _: &Path, branch_name: &str, worktree_folder: Option<&Path>, _: &str) -> Result<Option<BranchReview>> {
        let known = self.worktrees.lock().expect("scripted host lock").iter().any(|worktree| worktree.branch == branch_name);
        let dirty = self.dirty.lock().expect("scripted host lock").contains(branch_name);
        let ahead = self.ahead.lock().expect("scripted host lock").contains(branch_name);
        let conflicting = self.conflicting.lock().expect("scripted host lock").contains(branch_name);
        Ok(known.then(|| BranchReview {
            branch: branch_name.to_string(),
            commits_ahead: u32::from(ahead),
            changed_files: Vec::new(),
            merges_cleanly: Some(!conflicting),
            worktree_folder: worktree_folder.map(|folder| folder.to_string_lossy().into_owned()),
            has_unsaved_changes: dirty,
        }))
    }

    async fn status(&self, worktree: &Worktree) -> Result<WorktreeStatus> {
        let dirty = self.dirty.lock().expect("scripted host lock").contains(&worktree.branch);
        Ok(WorktreeStatus { has_unsaved_changes: dirty, commits_ahead: 0 })
    }

    async fn merge_branch(&self, _: &Path, branch_name: &str, base: &str) -> Result<MergeOutcome> {
        self.bases.lock().expect("scripted host lock").push(base.to_string());
        self.ahead.lock().expect("scripted host lock").remove(branch_name);
        self.merged.lock().expect("scripted host lock").push(branch_name.to_string());
        Ok(MergeOutcome::Merged)
    }

    async fn push_branch(&self, _: &Path, branch_name: &str) -> Result<PushOutcome> {
        if let Some(reason) = self.push_refusals.lock().expect("scripted host lock").get(branch_name) {
            return Ok(PushOutcome::Refused(reason.clone()));
        }
        let mut pushed = self.pushed.lock().expect("scripted host lock");
        if pushed.iter().any(|branch| branch == branch_name) {
            return Ok(PushOutcome::UpToDate);
        }
        pushed.push(branch_name.to_string());
        Ok(PushOutcome::Pushed)
    }
}
