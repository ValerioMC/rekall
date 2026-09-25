//! Logs a commit of a task's project folder against that task, or one of its steps: the tip by
//! default, or any commit named by its hash. Both the console and `rekall_record_commit` land
//! here: same lookup, same git read, same row. A hash already logged for that (task, step) pair
//! is returned as-is rather than duplicated. A new row is announced as a `commit-reference`
//! event so the console sees it without a reload.

use std::path::PathBuf;
use std::sync::LazyLock;

use regex::Regex;
use rekall_common::{jstr, Id, Instant, RekallError, Result};
use rekall_model::commit_reference::{self, COMMENT_MAX, DIFF_MAX};
use rekall_model::{task, task_step};
use rekall_repository::repository as repo;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};
use serde::Serialize;

use super::git::{GitLogReader, LogEntry};
use crate::{in_read, in_write, load, Ctx, DomainEvent, Tx};

/// How far back the picker looks.
pub const RECENT_LIMIT: usize = 30;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitReferenceView {
    pub id: Id,
    pub task_id: Id,
    pub step_id: Option<Id>,
    pub step_title: Option<String>,
    pub commit_hash: String,
    pub comment: String,
    pub in_context: bool,
    pub created_at: Instant,
}

impl CommitReferenceView {
    pub fn of(reference: &commit_reference::Model, step: Option<&task_step::Model>) -> Self {
        Self {
            id: reference.id,
            task_id: reference.task_id,
            step_id: step.map(|s| s.id),
            step_title: step.map(|s| s.title.clone()),
            commit_hash: reference.commit_hash.clone(),
            comment: reference.comment.clone(),
            in_context: reference.in_context,
            created_at: reference.created_at,
        }
    }

    async fn load(db: &impl sea_orm::ConnectionTrait, reference: &commit_reference::Model) -> Result<Self> {
        let step = match reference.step_id {
            Some(step_id) => repo::task_step::find_by_id(db, step_id).await?,
            None => None,
        };
        Ok(Self::of(reference, step.as_ref()))
    }
}

/// Emitted when a commit is logged against a task or step, whichever path logged it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitReferenceStreamEvent {
    pub task_id: Id,
    pub reference: CommitReferenceView,
}

/// One commit of a project's recent log, as the console lists it when picking one by hand.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentCommitView {
    pub hash: String,
    pub subject: String,
    pub committed_at: Instant,
}

impl RecentCommitView {
    fn of(entry: LogEntry) -> Self {
        Self { hash: entry.hash, subject: entry.subject, committed_at: entry.committed_at }
    }
}

#[derive(Clone)]
pub struct CommitReferenceService {
    ctx: Ctx,
    git: GitLogReader,
}

impl CommitReferenceService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx, git: GitLogReader }
    }

    /// Every commit logged so far, newest first.
    pub async fn find_all(&self) -> Result<Vec<CommitReferenceView>> {
        in_read!(&self.ctx, |tx| {
            let all = repo::commit_reference::find_all_by_order_by_created_at_desc(tx.db()).await?;
            let mut views = Vec::with_capacity(all.len());
            for reference in &all {
                views.push(CommitReferenceView::load(tx.db(), reference).await?);
            }
            Ok::<_, RekallError>(views)
        })
    }

    /// The recent log of the task's project folder, newest first.
    pub async fn recent_commits(&self, task_id: Id) -> Result<Vec<RecentCommitView>> {
        in_read!(&self.ctx, |tx| {
            let task = require_task(&tx, task_id).await?;
            let folder = repo_folder_of(&tx, &task).await?;
            let recent = self.git.recent(&folder, RECENT_LIMIT).await?;
            Ok::<_, RekallError>(recent.into_iter().map(RecentCommitView::of).collect())
        })
    }

    /// From the console: the tip, against the task or a step of it.
    pub async fn record_latest_commit(&self, task_id: Id, step_id: Option<Id>) -> Result<CommitReferenceView> {
        self.record_commit(task_id, step_id, None).await
    }

    /// From the console's picker: a commit chosen from the recent log or pasted by hand. A blank
    /// hash means the tip.
    pub async fn record_commit(&self, task_id: Id, step_id: Option<Id>, commit_hash: Option<&str>) -> Result<CommitReferenceView> {
        in_write!(&self.ctx, |tx| self.record_commit_in(&mut tx, task_id, step_id, commit_hash).await)
    }

    pub async fn record_commit_in(
        &self,
        tx: &mut Tx,
        task_id: Id,
        step_id: Option<Id>,
        commit_hash: Option<&str>,
    ) -> Result<CommitReferenceView> {
        let task = require_task(tx, task_id).await?;
        let step = resolve_step_by_id(tx, &task, step_id).await?;
        self.record(tx, &task, step, commit_hash).await
    }

    /// From `rekall_record_commit`: anchored the way every other tool anchors a task, with an
    /// optional step reference and an optional hash.
    pub async fn record_commit_by_anchor(
        &self,
        project_label: Option<&str>,
        task_label: &str,
        step_ref: Option<&str>,
        commit_hash: Option<&str>,
    ) -> Result<CommitReferenceView> {
        in_write!(&self.ctx, |tx| {
            let task = load::resolve_task(tx.db(), project_label, task_label).await?;
            let step = match step_ref.filter(|r| !jstr::is_blank(r)) {
                None => None,
                Some(reference) => Some(resolve_step_by_ref(&tx, &task, reference).await?),
            };
            self.record(&mut tx, &task, step, commit_hash).await
        })
    }

    async fn record(
        &self,
        tx: &mut Tx,
        task: &task::Model,
        step: Option<task_step::Model>,
        commit_hash: Option<&str>,
    ) -> Result<CommitReferenceView> {
        let folder = repo_folder_of(tx, task).await?;
        let commit = match commit_hash.filter(|h| !jstr::is_blank(h)) {
            None => self.git.head(&folder).await?,
            Some(hash) => self.git.commit(&folder, jstr::strip(hash)).await?,
        };
        let step_id = step.as_ref().map(|s| s.id);

        let existing = repo::commit_reference::find_by_task_id_and_commit_hash(tx.db(), task.id, &commit.hash).await?;
        if let Some(found) = existing.into_iter().find(|reference| reference.step_id == step_id) {
            return CommitReferenceView::load(tx.db(), &found).await;
        }

        let reference = commit_reference::Model {
            id: Id::random(),
            task_id: task.id,
            step_id,
            commit_hash: commit.hash.clone(),
            comment: truncated(&commit.subject),
            diff: truncated_diff(commit.diff.as_deref()),
            in_context: false,
            created_at: self.ctx.now(),
        };
        reference.validate()?;
        reference.clone().into_active_model().insert(tx.db()).await?;
        let logged = CommitReferenceView::of(&reference, step.as_ref());
        tx.publish(DomainEvent::CommitReference(CommitReferenceStreamEvent {
            task_id: logged.task_id,
            reference: logged.clone(),
        }));
        Ok(logged)
    }

    /// The diff stored for one logged commit, read on demand: the list stays light.
    pub async fn diff_for(&self, id: Id) -> Result<Option<String>> {
        in_read!(&self.ctx, |tx| {
            let reference = repo::commit_reference::find_by_id(tx.db(), id)
                .await?
                .ok_or_else(|| RekallError::not_found("commit reference", id))?;
            Ok::<_, RekallError>(reference.diff)
        })
    }

    /// Whether a logged commit rides along with `rekall_context`. A missing flag reads as off, so
    /// a body that omits it never turns a commit on by accident.
    pub async fn set_in_context(&self, id: Id, in_context: Option<bool>) -> Result<CommitReferenceView> {
        in_write!(&self.ctx, |tx| {
            let mut reference = repo::commit_reference::find_by_id(tx.db(), id)
                .await?
                .ok_or_else(|| RekallError::not_found("commit reference", id))?;
            reference.in_context = in_context == Some(true);
            reference.clone().into_active_model().reset_all().update(tx.db()).await?;
            CommitReferenceView::load(tx.db(), &reference).await
        })
    }

    /// Removes the link between a commit and what it was logged against.
    pub async fn delete(&self, id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            if repo::commit_reference::find_by_id(tx.db(), id).await?.is_none() {
                return Err(RekallError::not_found("commit reference", id));
            }
            commit_reference::Entity::delete_by_id(id).exec(tx.db()).await?;
            Ok(())
        })
    }
}

async fn require_task(tx: &Tx, task_id: Id) -> Result<task::Model> {
    repo::task::find_by_id(tx.db(), task_id)
        .await?
        .ok_or_else(|| RekallError::not_found("task", task_id))
}

async fn repo_folder_of(tx: &Tx, task: &task::Model) -> Result<PathBuf> {
    let project = load::project_of(tx.db(), task).await?;
    match project.repo_folder.as_deref().filter(|f| !jstr::is_blank(f)) {
        None => Err(RekallError::illegal("Set this project's folder on its page before logging a commit here.")),
        Some(folder) => Ok(PathBuf::from(jstr::strip(folder))),
    }
}

async fn resolve_step_by_id(tx: &Tx, task: &task::Model, step_id: Option<Id>) -> Result<Option<task_step::Model>> {
    let Some(step_id) = step_id else {
        return Ok(None);
    };
    let step = repo::task_step::find_by_id(tx.db(), step_id)
        .await?
        .ok_or_else(|| RekallError::not_found("step", step_id))?;
    if step.task_id != task.id {
        return Err(RekallError::illegal("That step does not belong to this task."));
    }
    Ok(Some(step))
}

async fn resolve_step_by_ref(tx: &Tx, task: &task::Model, reference: &str) -> Result<task_step::Model> {
    static DIGITS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d+$").unwrap());
    let ordered = load::steps_of(tx.db(), task.id).await?;
    if ordered.is_empty() {
        return Err(RekallError::illegal("That task has no steps to log this commit against."));
    }
    let trimmed = jstr::strip(reference);
    if DIGITS.is_match(trimmed) {
        let one_based: i64 = trimmed
            .parse::<i32>()
            .map_err(|_| RekallError::illegal(format!("For input string: \"{trimmed}\"")))?
            .into();
        if one_based < 1 || one_based > ordered.len() as i64 {
            return Err(RekallError::illegal(format!(
                "There is no step {one_based}. The checklist has {}.",
                ordered.len()
            )));
        }
        return Ok(ordered[(one_based - 1) as usize].clone());
    }
    let by_title: Vec<&task_step::Model> = ordered
        .iter()
        .filter(|step| jstr::equals_ignore_case(jstr::strip(&step.title), trimmed))
        .collect();
    if by_title.len() == 1 {
        return Ok(by_title[0].clone());
    }
    Err(RekallError::illegal(format!(
        "No step matches '{reference}'. Pass its number or its exact title."
    )))
}

fn truncated(subject: &str) -> String {
    let text = jstr::strip(subject);
    if text.is_empty() {
        return "(no commit message)".into();
    }
    if jstr::len(text) > COMMENT_MAX {
        format!("{}…", jstr::prefix(text, COMMENT_MAX - 1))
    } else {
        text.to_string()
    }
}

fn truncated_diff(diff: Option<&str>) -> Option<String> {
    let diff = diff.filter(|d| !jstr::is_blank(d))?;
    Some(if jstr::len(diff) > DIFF_MAX {
        format!("{}\n…(diff truncated)", jstr::prefix(diff, DIFF_MAX))
    } else {
        diff.to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_overlong_subject_is_cut_to_the_comment_cap() {
        let cut = truncated(&"x".repeat(COMMENT_MAX + 50));
        assert_eq!(jstr::len(&cut), COMMENT_MAX);
        assert!(cut.ends_with('…'));
        assert_eq!(truncated("   "), "(no commit message)");
    }

    #[test]
    fn an_overlong_diff_is_truncated_rather_than_stored_whole() {
        let cut = truncated_diff(Some(&"+".repeat(DIFF_MAX + 500))).unwrap();
        assert_eq!(jstr::len(&cut), DIFF_MAX + jstr::len("\n…(diff truncated)"));
        assert!(cut.ends_with("…(diff truncated)"));
        assert_eq!(truncated_diff(None), None);
        assert_eq!(truncated_diff(Some("  ")), None);
    }
}
