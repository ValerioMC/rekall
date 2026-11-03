use std::path::PathBuf;

use rekall_kernel::{Id, RekallError, Result};
use rekall_store::repository as repo;
use rekall_store::step_snapshot;
use tracing::warn;

use crate::commit::{GitCommitter, GitRepositoryInspector};
use crate::{in_write, load, Ctx};

/// Remembers which files were already modified when a step, or a task with no steps, started
/// running, so a later commit can tell them from the session's own work. Only auto-commit
/// projects keep one. A failure is logged and never stops the work.
#[derive(Clone)]
pub struct StepSnapshotService {
    ctx: Ctx,
    inspector: GitRepositoryInspector,
    committer: GitCommitter,
}

impl StepSnapshotService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx, inspector: GitRepositoryInspector, committer: GitCommitter }
    }

    /// `step_id` is `None` for a task with no steps. The snapshot already kept for the same step
    /// or task is replaced, since a step starts again after it is sent back.
    pub async fn take(&self, task_id: Id, step_id: Option<Id>) {
        if let Err(failed) = self.keep(task_id, step_id).await {
            warn!(%failed, %task_id, "Could not snapshot the working tree");
        }
    }

    async fn keep(&self, task_id: Id, step_id: Option<Id>) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let Some(task) = repo::task::find_by_id(tx.db(), task_id).await? else {
                return Ok(());
            };
            let project = load::project_of(tx.db(), &task).await?;
            if !project.auto_commit {
                return Ok(());
            }
            let status = self.inspector.inspect(project.repo_folder.as_deref()).await;
            let Some(folder) = status.folder.filter(|_| status.repository) else {
                return Err(RekallError::illegal(format!(
                    "the project's folder ({}) is not a git repository.",
                    project.repo_folder.as_deref().unwrap_or("none set")
                )));
            };
            let taken = self.committer.work_tree_snapshot(&PathBuf::from(folder)).await?;
            let header = step_snapshot::Model { id: Id::random(), task_id, step_id, taken_at: self.ctx.now() };
            let files = taken.entries.into_iter().map(|entry| (entry.path, entry.content_hash)).collect();
            repo::step_snapshot::replace(tx.db(), header, files).await
        })
    }
}
