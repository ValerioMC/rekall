use std::path::Path;

use rekall_kernel::{Id, RekallError, Result};
use rekall_store::commit_reference;
use rekall_store::repository as repo;
use tracing::warn;

use crate::commit::model::unified_diff::UnifiedDiff;
use crate::commit::model::repository_target::RepositoryTarget;
use crate::commit::view::{CommitChangesView, RepositoryChangesView, TaskChangesView, WorkingTreeChangesView};
use crate::commit::{GitLogReader, GitWorkingTreeReader};
use crate::{in_read, load, Ctx};

/// A patch longer than this is cut, so one huge commit cannot fill the console's memory.
const PATCH_MAX: usize = 2_000_000;

/// Reads what a task changed in each repository of its project: each logged commit's patch,
/// oldest first, and the working tree. A commit git no longer knows falls back to the diff stored when it was
/// logged. Never writes.
#[derive(Clone)]
pub struct TaskChangesService {
    ctx: Ctx,
    log: GitLogReader,
    working_tree: GitWorkingTreeReader,
}

impl TaskChangesService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx, log: GitLogReader, working_tree: GitWorkingTreeReader }
    }

    pub async fn of_task(&self, task_id: Id) -> Result<TaskChangesView> {
        in_read!(&self.ctx, |tx| {
            let task = repo::task::require(tx.db(), task_id).await?;
            let project = load::project_of(tx.db(), &task).await?;
            let components = repo::project_component::list_of_project(tx.db(), project.id).await?;
            let mut references = repo::commit_reference::list_of_task(tx.db(), task.id).await?;
            references.sort_by_key(|reference| reference.created_at);
            let mut repositories = Vec::new();
            for target in RepositoryTarget::all_of(&task, &project, &components) {
                let changes = self.repository_changes(&target, &references).await;
                if !changes.is_empty() {
                    repositories.push(changes);
                }
            }
            Ok::<_, RekallError>(TaskChangesView { repositories })
        })
    }

    async fn repository_changes(&self, target: &RepositoryTarget, references: &[commit_reference::Model]) -> RepositoryChangesView {
        let mut commits = Vec::new();
        for reference in references.iter().filter(|reference| reference.component_id == target.component_id) {
            commits.push(self.commit_changes(target.folder.as_deref(), reference).await);
        }
        let working_tree = match target.folder.as_deref().filter(|_| target.reads_own_working_tree) {
            Some(folder) => self.working_tree_changes(folder).await,
            None => None,
        };
        RepositoryChangesView { component_id: target.component_id, name: target.name.clone(), commits, working_tree }
    }

    async fn commit_changes(&self, folder: Option<&Path>, reference: &commit_reference::Model) -> CommitChangesView {
        let patch = self.patch_of(folder, reference).await;
        let available = patch.is_some();
        let (kept, truncated) = capped(patch.unwrap_or_default());
        CommitChangesView {
            reference_id: reference.id,
            commit_hash: reference.commit_hash.clone(),
            subject: reference.comment.clone(),
            step_id: reference.step_id,
            logged_at: reference.created_at,
            available,
            truncated,
            files: UnifiedDiff::parse(&kept),
        }
    }

    async fn patch_of(&self, folder: Option<&Path>, reference: &commit_reference::Model) -> Option<String> {
        if let Some(folder) = folder {
            match self.log.patch(folder, &reference.commit_hash).await {
                Ok(patch) => return Some(patch),
                Err(failed) => warn!(%failed, commit = %reference.commit_hash, "Reading the stored diff: git could not give the commit"),
            }
        }
        reference.diff.clone()
    }

    async fn working_tree_changes(&self, folder: &Path) -> Option<WorkingTreeChangesView> {
        let patch = match self.working_tree.patch(folder).await {
            Ok(patch) => patch,
            Err(failed) => {
                warn!(%failed, "Could not read the working tree");
                return None;
            }
        };
        let (kept, truncated) = capped(patch);
        let files = UnifiedDiff::parse(&kept);
        if files.is_empty() {
            return None;
        }
        Some(WorkingTreeChangesView { truncated, files })
    }
}

fn capped(patch: String) -> (String, bool) {
    match patch.char_indices().nth(PATCH_MAX) {
        Some((cut, _)) => (patch[..cut].to_string(), true),
        None => (patch, false),
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/commit/service/task_changes_service_tests.rs"]
mod tests;
