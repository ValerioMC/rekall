use std::path::PathBuf;

use rekall_common::{Id, RekallError, Result};
use rekall_model::{task, task_step};
use rekall_repository::repository as repo;
use tracing::{info, warn};

use super::git::{display_path, GitCommitter, GitRepositoryInspector};
use super::{CommitMessageGenerator, Subject};
use super::CommitReferenceService;
use crate::{in_write, load, Ctx, Tx};

use super::AutoCommitOutcome;

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
        let status = self.inspector.inspect(project.repo_folder.as_deref()).await;
        if !status.repository {
            return Ok(AutoCommitOutcome::failed(format!(
                "this project's folder ({}) is not a git repository.",
                project.repo_folder.as_deref().unwrap_or("null")
            )));
        }
        let status_folder = status.folder.clone().unwrap_or_default();
        if status.user_email.is_none() {
            return Ok(AutoCommitOutcome::failed(format!(
                "git has no user.email in {status_folder}; set it with `git config --global user.email`."
            )));
        }
        let folder = PathBuf::from(&status_folder);
        let shown = display_path(&folder);
        let attempt: Result<AutoCommitOutcome> = async {
            let changes = self.committer.pending_changes(&folder).await?;
            if changes.is_empty() {
                return Ok(AutoCommitOutcome::skipped(format!("nothing to commit in {shown}, so nothing was logged.")));
            }
            let subject = self.subject_of(tx, task, &project.label, step).await?;
            let message = CommitMessageGenerator::generate(&subject, &changes, session_message);
            let hash = self.committer.commit_all(&folder, &message).await?;
            info!(
                "Auto-committed {hash} in {shown} for task {} step {}",
                task.label,
                step.map(|s| s.title.as_str()).unwrap_or("-")
            );
            let logged = self.references.record_commit_in(tx, task.id, step.map(|s| s.id), Some(&hash)).await?;
            Ok(AutoCommitOutcome::committed(logged))
        }
        .await;
        match attempt {
            Err(RekallError::IllegalArgument(reason)) => {
                warn!("Auto-commit refused in {shown} for task {}: {reason}", task.label);
                Ok(AutoCommitOutcome::failed(reason))
            }
            other => other,
        }
    }

    async fn subject_of(&self, tx: &Tx, task: &task::Model, project_label: &str, step: Option<&task_step::Model>) -> Result<Subject> {
        let anchor = format!("project:{project_label} task:{}", task.label);
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
