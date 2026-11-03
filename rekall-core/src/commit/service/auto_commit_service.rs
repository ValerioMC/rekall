use std::path::{Path, PathBuf};

use rekall_kernel::{Id, RekallError, Result};
use rekall_store::{project, task, task_step};
use rekall_store::repository as repo;
use tracing::{info, warn};

use crate::commit::git::{display_path, GitCommitter, GitRepositoryInspector};
use crate::commit::PendingChange;
use crate::commit::model::session_changes::SessionChanges;
use crate::commit::{CommitMessageGenerator, Subject};
use crate::commit::CommitReferenceService;
use crate::{in_write, load, Ctx, Tx};

use crate::commit::AutoCommitOutcome;

#[derive(Clone)]
pub struct AutoCommitService {
    ctx: Ctx,
    inspector: GitRepositoryInspector,
    committer: GitCommitter,
    references: CommitReferenceService,
}

impl AutoCommitService {
    pub fn new(ctx: Ctx, references: CommitReferenceService) -> Self {
        Self { ctx, inspector: GitRepositoryInspector, committer: GitCommitter, references }
    }

    /// `session_message` is the commit message the claiming session wrote, subject line first,
    /// or `None` to have one derived from the step.
    pub async fn after_step_claim(&self, task_id: Id, step_id: Id, session_message: Option<&str>) -> Result<AutoCommitOutcome> {
        in_write!(&self.ctx, |tx| {
            let task = repo::task::find_by_id(tx.db(), task_id).await?;
            let step = repo::task_step::find_by_id(tx.db(), step_id).await?;
            let (Some(task), Some(step)) = (task, step) else {
                return Ok(AutoCommitOutcome::not_applicable());
            };
            self.commit(&mut tx, &task, Some(&step), session_message).await
        })
    }

    /// `session_message` is the commit message the session wrote with the wrapup, or `None` to
    /// have one derived from the wrapup itself.
    pub async fn after_wrapup(&self, task_id: Id, session_message: Option<&str>) -> Result<AutoCommitOutcome> {
        in_write!(&self.ctx, |tx| {
            let Some(task) = repo::task::find_by_id(tx.db(), task_id).await? else {
                return Ok(AutoCommitOutcome::not_applicable());
            };
            if !load::review_active(tx.db(), task_id).await? {
                return Ok(AutoCommitOutcome::not_applicable());
            }
            self.commit(&mut tx, &task, None, session_message).await
        })
    }

    async fn commit(
        &self,
        tx: &mut Tx,
        task: &task::Model,
        step: Option<&task_step::Model>,
        session_message: Option<&str>,
    ) -> Result<AutoCommitOutcome> {
        let project = load::project_of(tx.db(), task).await?;
        if !project.auto_commit {
            return Ok(AutoCommitOutcome::not_applicable());
        }
        let folder = match self.committable_folder(&project).await {
            Ok(folder) => folder,
            Err(refusal) => return Ok(refusal),
        };
        let attempt = self.commit_pending(tx, task, &project.label, step, session_message, &folder).await;
        match attempt {
            Err(RekallError::IllegalArgument(reason)) => {
                warn!(%reason, folder = %display_path(&folder), task = %task.label, "Auto-commit refused");
                Ok(AutoCommitOutcome::failed(reason))
            }
            other => other,
        }
    }

    /// The repository folder git can commit in, or the failed outcome saying why there is none.
    async fn committable_folder(&self, project: &project::Model) -> std::result::Result<PathBuf, AutoCommitOutcome> {
        let status = self.inspector.inspect(project.repo_folder.as_deref()).await;
        if !status.repository {
            return Err(AutoCommitOutcome::failed(format!(
                "this project's folder ({}) is not a git repository.",
                project.repo_folder.as_deref().unwrap_or("none set")
            )));
        }
        let status_folder = status.folder.unwrap_or_default();
        if status.user_email.is_none() {
            return Err(AutoCommitOutcome::failed(format!(
                "git has no user.email in {status_folder}; set it with `git config --global user.email`."
            )));
        }
        Ok(PathBuf::from(status_folder))
    }

    async fn commit_pending(
        &self,
        tx: &mut Tx,
        task: &task::Model,
        project_label: &str,
        step: Option<&task_step::Model>,
        session_message: Option<&str>,
        folder: &Path,
    ) -> Result<AutoCommitOutcome> {
        let shown = display_path(folder);
        let pending = self.committer.pending_changes(folder).await?;
        if pending.is_empty() {
            return Ok(AutoCommitOutcome::skipped(format!("nothing to commit in {shown}, so nothing was logged.")));
        }
        let snapshot = self.snapshot_files(tx, task, step).await?;
        let changes = self.sort_changes(folder, pending, snapshot.as_deref()).await?;
        let left_out: Vec<String> = changes.left_out.iter().map(|change| change.path.clone()).collect();
        let missing_snapshot = snapshot.is_none();
        if changes.own.is_empty() {
            return Ok(AutoCommitOutcome::skipped("nothing of this work to commit, so nothing was logged.".to_string())
                .leaving_out(left_out));
        }
        let subject = self.subject_of(tx, task, project_label, step).await?;
        let message = CommitMessageGenerator::generate(&subject, &changes.own, session_message);
        let hash = self.committer.commit_paths(folder, &changes.own, &message).await?;
        info!(%hash, folder = %shown, task = %task.label, step = step.map(|s| s.title.as_str()).unwrap_or("-"), "Auto-committed");
        let logged = self.references.record_commit_in(tx, task.id, step.map(|s| s.id), Some(&hash)).await?;
        let outcome = AutoCommitOutcome::committed(logged).leaving_out(left_out);
        Ok(if missing_snapshot { outcome.without_snapshot() } else { outcome })
    }

    async fn snapshot_files(
        &self,
        tx: &Tx,
        task: &task::Model,
        step: Option<&task_step::Model>,
    ) -> Result<Option<Vec<(String, Option<String>)>>> {
        let Some(snapshot) = repo::step_snapshot::find(tx.db(), task.id, step.map(|s| s.id)).await? else {
            return Ok(None);
        };
        let files = repo::step_snapshot::files_of(tx.db(), snapshot.id).await?;
        Ok(Some(files.into_iter().map(|file| (file.path, file.content_hash)).collect()))
    }

    async fn sort_changes(
        &self,
        folder: &Path,
        pending: Vec<PendingChange>,
        snapshot: Option<&[(String, Option<String>)]>,
    ) -> Result<SessionChanges> {
        let Some(snapshot) = snapshot else {
            return Ok(SessionChanges::everything(pending));
        };
        let hashes = self.committer.content_hashes_of(folder, &pending).await?;
        Ok(SessionChanges::split(pending, hashes, snapshot))
    }

    async fn subject_of(&self, tx: &Tx, task: &task::Model, project_label: &str, step: Option<&task_step::Model>) -> Result<Subject> {
        let anchor = task.anchor(project_label);
        Ok(match step {
            None => Subject {
                title: task.title.clone(),
                anchor,
                step_number: None,
                description: load::wrapup_of(tx.db(), task.id).await?.map(|w| w.body_markdown),
            },
            Some(step) => Subject {
                title: step.title.clone(),
                anchor,
                step_number: Some(step.position + 1),
                description: step.body_markdown.clone(),
            },
        })
    }
}
