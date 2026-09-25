//! Commits a project's repo folder when a session claims work on it, if the project asks for it.
//! A step claimed through `rekall_step` commits against that step; a Claude-authored wrapup on a
//! stepless task commits against the task, since that wrapup is what claims it. A task with a
//! checklist never commits on its wrapup: its steps carry the claims. Nothing here fails the
//! claim that triggered it: that has already been written and stays written.

use std::path::PathBuf;

use rekall_common::{Id, RekallError, Result};
use rekall_model::{task, task_step};
use rekall_repository::repository as repo;
use tracing::{info, warn};

use super::git::{display_path, GitCommitter, GitRepositoryInspector};
use super::message::{CommitMessageGenerator, Subject};
use super::reference::{CommitReferenceService, CommitReferenceView};
use crate::{in_write, load, Ctx, Tx};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutoCommitStatus {
    /// The project does not auto-commit, or this claim is not one that commits.
    Off,
    /// The project auto-commits, but there was nothing in the working tree to commit.
    Skipped,
    /// A commit was made and logged against the step or task.
    Committed,
    /// The project auto-commits, but git refused; `detail` says why.
    Failed,
}

/// What an automatic commit came to, for the session that triggered it.
#[derive(Clone, Debug)]
pub struct AutoCommitOutcome {
    pub status: AutoCommitStatus,
    pub detail: Option<String>,
    pub reference: Option<CommitReferenceView>,
}

impl AutoCommitOutcome {
    fn not_applicable() -> Self {
        Self { status: AutoCommitStatus::Off, detail: None, reference: None }
    }

    fn skipped(detail: String) -> Self {
        Self { status: AutoCommitStatus::Skipped, detail: Some(detail), reference: None }
    }

    fn committed(reference: CommitReferenceView) -> Self {
        Self { status: AutoCommitStatus::Committed, detail: None, reference: Some(reference) }
    }

    fn failed(detail: String) -> Self {
        Self { status: AutoCommitStatus::Failed, detail: Some(detail), reference: None }
    }

    pub fn off(&self) -> bool {
        self.status == AutoCommitStatus::Off
    }

    /// The line a tool appends to its report: nothing when the project does not auto-commit.
    pub fn describe(&self) -> String {
        let detail = self.detail.as_deref().unwrap_or("null");
        match self.status {
            AutoCommitStatus::Off => String::new(),
            AutoCommitStatus::Skipped => format!("Auto-commit: {detail}"),
            AutoCommitStatus::Failed => format!("Auto-commit failed: {detail} The claim stands; commit and log by hand."),
            AutoCommitStatus::Committed => {
                let reference = self.reference.as_ref().expect("a commit has its reference");
                let short = &reference.commit_hash[..reference.commit_hash.len().min(7)];
                let against = match &reference.step_title {
                    None => "the task".to_string(),
                    Some(title) => format!("\"{title}\""),
                };
                format!(
                    "Auto-committed `{short}` — {} — and logged it against {against}. Do not commit this work again.",
                    reference.comment
                )
            }
        }
    }
}

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

