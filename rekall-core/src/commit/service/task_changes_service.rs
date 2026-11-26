use std::{path::Path, sync::Arc, time::Instant};

use rekall_kernel::{Id, RekallError, Result};
use rekall_store::commit_reference;
use rekall_store::repository as repo;
use tokio::sync::Mutex;
use tracing::warn;

use crate::commit::model::indexed_patch::IndexedPatch;
use crate::commit::model::repository_target::RepositoryTarget;
use crate::commit::model::task_changes_snapshot::TaskChangesSnapshot;
use crate::commit::model::task_changes_snapshots::TaskChangesSnapshots;
use crate::commit::model::unified_diff::UnifiedDiff;
use crate::commit::view::{ChangeFileHunksView, CommitChangesView, RepositoryChangesView, TaskChangesIndexView, TaskChangesView, WorkingTreeChangesView};
use crate::commit::{ChangeFileKey, GitLogReader, GitWorkingTreeReader, TaskChangesScope};
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
    snapshots: Arc<Mutex<TaskChangesSnapshots>>,
}

impl TaskChangesService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx, log: GitLogReader, working_tree: GitWorkingTreeReader, snapshots: Arc::default() }
    }

    pub async fn of_task(&self, task_id: Id) -> Result<TaskChangesView> {
        let repositories = self.read_repositories(task_id, |patch| {
            let files = UnifiedDiff::parse(&patch);
            (!files.is_empty()).then_some(files)
        }).await?;
        Ok(TaskChangesView { repositories })
    }

    pub async fn index(&self, task_id: Id) -> Result<TaskChangesIndexView> {
        let repositories = self.read_repositories(task_id, |patch| IndexedPatch::read(patch).map(Arc::new)).await?;
        let snapshot = TaskChangesSnapshot::new(task_id, repositories);
        let index = snapshot.index();
        self.snapshots.lock().await.insert(Instant::now(), snapshot);
        Ok(index)
    }

    pub async fn hunks(&self, task_id: Id, snapshot_id: Id, scope: TaskChangesScope, files: &[ChangeFileKey]) -> Result<Vec<ChangeFileHunksView>> {
        let snapshot = self.snapshots.lock().await.require(Instant::now(), task_id, snapshot_id)?;
        snapshot.hunks(scope, files)
    }

    async fn read_repositories<Files: Default>(
        &self,
        task_id: Id,
        read_files: impl Fn(String) -> Option<Files>,
    ) -> Result<Vec<RepositoryChangesView<Files>>> {
        in_read!(&self.ctx, |tx| {
            let task = repo::task::require(tx.db(), task_id).await?;
            let project = load::project_of(tx.db(), &task).await?;
            let components = repo::project_component::list_of_project(tx.db(), project.id).await?;
            let mut references = repo::commit_reference::list_of_task(tx.db(), task.id).await?;
            references.sort_by_key(|reference| (reference.created_at, reference.id));
            let mut repositories = Vec::new();
            for target in RepositoryTarget::all_of(&task, &project, &components) {
                let changes = self.repository_changes(&target, &references).await.map_files(&read_files);
                if !changes.is_empty() {
                    repositories.push(changes);
                }
            }
            Ok::<_, RekallError>(repositories)
        })
    }

    async fn repository_changes(&self, target: &RepositoryTarget, references: &[commit_reference::Model]) -> RepositoryChangesView<String> {
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

    async fn commit_changes(&self, folder: Option<&Path>, reference: &commit_reference::Model) -> CommitChangesView<String> {
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
            files: kept,
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

    async fn working_tree_changes(&self, folder: &Path) -> Option<WorkingTreeChangesView<String>> {
        let patch = match self.working_tree.patch(folder).await {
            Ok(patch) => patch,
            Err(failed) => {
                warn!(%failed, "Could not read the working tree");
                return None;
            }
        };
        let (files, truncated) = capped(patch);
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
