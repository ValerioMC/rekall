use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rekall_kernel::{jstr, Id, Result};
use rekall_store::repository as repo;
use tracing::warn;

use crate::worktree::{BranchReview, MergeOutcome, OrphanReclaimService, Worktree, WorktreeHost, WorktreeLayout};
use crate::{in_read, in_write, load, Ctx};

/// What taking a task's worktree away did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Release {
    Removed,
    /// The worktree has unsaved changes and was left as it was.
    KeptWithChanges,
    NoWorktree,
}

/// What bringing an accepted task's branch into the project came to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Merge {
    Merged,
    /// The branch holds nothing the project's `HEAD` lacks.
    AlreadyMerged,
    /// The worktree has unsaved changes, so the branch is not the whole work yet.
    KeptWithChanges,
    /// The branch conflicts with the project or git refused; nothing was changed.
    Refused(String),
    /// The task has no branch, or its project no folder.
    NothingToMerge,
}

/// Gives a task a worktree to work in, and keeps the tasks and the worktrees on disk in step.
/// The task row names its worktree (`worktree_folder`), which is what makes its session, its
/// checks and its commit work there.
#[derive(Clone)]
pub struct TaskWorktreeService {
    ctx: Ctx,
    host: Arc<dyn WorktreeHost>,
    layout: WorktreeLayout,
}

impl TaskWorktreeService {
    pub fn new(ctx: Ctx, host: Arc<dyn WorktreeHost>, layout: WorktreeLayout) -> Self {
        Self { ctx, host, layout }
    }

    /// The task's worktree: the one it already has when its folder is there, else a new one on
    /// `rekall/<task-label>` (the branch is reused when it exists), named on the task.
    pub async fn ensure(&self, task_id: Id) -> Result<PathBuf> {
        let (task, project_folder, project_label) = self.read(task_id).await?;
        if let Some(folder) = task.worktree_folder.as_deref().map(PathBuf::from).filter(|folder| folder.is_dir()) {
            return Ok(folder);
        }
        let project_folder = project_folder.ok_or_else(|| rekall_kernel::RekallError::illegal("Set this project's folder on its page before working here."))?;
        let branch = self.layout.branch_for(&task.label)?;
        let folder = self.layout.folder_for(&project_label, &task.label)?;
        let worktree = self.host.create(Path::new(&project_folder), &branch, &folder).await?;
        self.name_on_task(task_id, Some(worktree.folder.to_string_lossy().into_owned())).await?;
        Ok(worktree.folder)
    }

    /// Takes the task's worktree away once its work is accepted, or when a person asks. A worktree
    /// with unsaved changes is left alone; the branch and its commits always stay.
    pub async fn release(&self, task_id: Id) -> Result<Release> {
        let (task, project_folder, _) = self.read(task_id).await?;
        let Some(folder) = task.worktree_folder.clone() else { return Ok(Release::NoWorktree) };
        let Some(project_folder) = project_folder else { return Ok(Release::NoWorktree) };
        let worktree = Worktree { project_folder: PathBuf::from(project_folder), folder: PathBuf::from(&folder), branch: self.layout.branch_for(&task.label)? };
        if worktree.folder.is_dir() {
            if self.host.status(&worktree).await?.has_unsaved_changes {
                return Ok(Release::KeptWithChanges);
            }
            self.host.remove(&worktree).await?;
        }
        self.name_on_task(task_id, None).await?;
        Ok(Release::Removed)
    }

    /// Merges the task's branch into the project folder's current `HEAD` once its work is accepted.
    /// A branch with unsaved work in its worktree, or one that conflicts, is left as it is.
    pub async fn merge_accepted(&self, task_id: Id) -> Result<Merge> {
        let Some(review) = self.review_branch(task_id).await? else { return Ok(Merge::NothingToMerge) };
        if review.has_unsaved_changes {
            return Ok(Merge::KeptWithChanges);
        }
        if review.commits_ahead == 0 {
            return Ok(Merge::AlreadyMerged);
        }
        if review.merges_cleanly == Some(false) {
            return Ok(Merge::Refused(format!("{} conflicts with the project's current branch", review.branch)));
        }
        let (_, project_folder, _) = self.read(task_id).await?;
        let Some(project_folder) = project_folder else { return Ok(Merge::NothingToMerge) };
        match self.host.merge_branch(Path::new(&project_folder), &review.branch).await? {
            MergeOutcome::Merged => Ok(Merge::Merged),
            MergeOutcome::Refused(reason) => Ok(Merge::Refused(reason)),
        }
    }

    /// What the task's branch holds, while the branch exists: the worktree or not.
    pub async fn review_branch(&self, task_id: Id) -> Result<Option<BranchReview>> {
        let (task, project_folder, _) = self.read(task_id).await?;
        let Some(project_folder) = project_folder else { return Ok(None) };
        let branch = self.layout.branch_for(&task.label)?;
        let worktree = task.worktree_folder.as_deref().map(Path::new).filter(|folder| folder.is_dir());
        self.host.review_branch(Path::new(&project_folder), &branch, worktree).await
    }

    /// After a restart: a task whose worktree folder is gone forgets it (its branch stays and the
    /// next run makes the folder again), and the worktrees no task names are removed when clean
    /// and reported when not.
    pub async fn reconcile(&self) -> Result<()> {
        self.forget_missing_folders().await?;
        let (projects, tasks) = in_read!(&self.ctx, |tx| Ok::<_, rekall_kernel::RekallError>((
            repo::project::list_oldest_first(tx.db()).await?,
            repo::task::list_with_worktree(tx.db()).await?
        )))?;
        let reclaim = OrphanReclaimService::new(self.host.clone());
        for project in projects {
            let Some(folder) = project.repo_folder.as_deref().filter(|folder| !jstr::is_blank(folder)) else { continue };
            let named: HashSet<String> = tasks
                .iter()
                .filter(|task| task.project_id == project.id)
                .filter_map(|task| self.layout.branch_for(&task.label).ok())
                .collect();
            if let Err(failure) = reclaim.reclaim_orphans(Path::new(jstr::strip(folder)), &named).await {
                warn!(%failure, project = %project.label, "Could not reclaim the worktrees of a project");
            }
        }
        Ok(())
    }

    async fn read(&self, task_id: Id) -> Result<(rekall_store::task::Model, Option<String>, String)> {
        in_read!(&self.ctx, |tx| {
            let task = load::task_or_unknown(tx.db(), task_id).await?;
            let project = load::project_of(tx.db(), &task).await?;
            let folder = project.repo_folder.clone().filter(|folder| !jstr::is_blank(folder));
            Ok((task, folder, project.label))
        })
    }

    async fn forget_missing_folders(&self) -> Result<()> {
        let attached = in_read!(&self.ctx, |tx| repo::task::list_with_worktree(tx.db()).await)?;
        for task in attached {
            let there = task.worktree_folder.as_deref().is_some_and(|folder| Path::new(folder).is_dir());
            if !there {
                self.name_on_task(task.id, None).await?;
            }
        }
        Ok(())
    }

    async fn name_on_task(&self, task_id: Id, folder: Option<String>) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let before = load::task_or_unknown(tx.db(), task_id).await?;
            let mut task = before.clone();
            task.worktree_folder = folder;
            repo::update_if_changed(tx.db(), &before, task, self.ctx.now()).await?;
            Ok(())
        })
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/worktree/service/task_worktree_service_tests.rs"]
mod tests;
