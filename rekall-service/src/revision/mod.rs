//! Earlier versions of a task's wrapup and description, so replacing or deleting one is never the
//! end of it. Whatever is about to overwrite the text calls `keep` with the text about to go:
//!
//! - Blank text, or text identical to the newest revision, is not kept.
//! - A hand edit in the console autosaves as it is typed, so a hand edit over hand-written text
//!   keeps one revision per ten-minute window: the version from before the editing started. A
//!   hand edit over a session's text is always kept, since that text was nobody's draft.
//! - A session's write, a deletion and a restore are discrete events and are always kept.
//! - Only the newest thirty revisions of each text are kept per task.

use chrono::TimeDelta;
use rekall_common::{Id, Instant, RekallError, Result};
use rekall_model::task_revision;
use rekall_model::{RevisionKind, WrapupAuthor};
use rekall_repository::repository as repo;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};
use serde::Serialize;

use crate::{in_read, Ctx, Tx};

pub const KEPT_PER_KIND: usize = 30;

pub fn hand_edit_window() -> TimeDelta {
    TimeDelta::minutes(10)
}

/// What is about to replace a task's text, which decides whether the text it replaces is kept.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RevisionTrigger {
    /// The console's editor saving as someone types: kept once per editing window.
    HandEdit,
    /// A session writing over MCP: always kept.
    ClaudeWrite,
    /// The text is being removed: always kept.
    Deletion,
    /// An earlier revision is being written back: always kept, so the restore can itself be undone.
    Restore,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskRevisionView {
    pub id: Id,
    pub task_id: Id,
    pub kind: RevisionKind,
    pub body_markdown: String,
    pub written_by: Option<WrapupAuthor>,
    pub written_at: Option<Instant>,
    pub replaced_at: Instant,
}

impl TaskRevisionView {
    pub fn of(revision: &task_revision::Model) -> Self {
        Self {
            id: revision.id,
            task_id: revision.task_id,
            kind: revision.kind,
            body_markdown: revision.body_markdown.clone(),
            written_by: revision.written_by,
            written_at: revision.written_at,
            replaced_at: revision.created_at,
        }
    }
}

/// What a restore wrote back: which text of which task, and its new current body.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoredRevision {
    pub task_id: Id,
    pub kind: RevisionKind,
    pub body_markdown: String,
}

#[derive(Clone)]
pub struct TaskRevisionService {
    ctx: Ctx,
}

impl TaskRevisionService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    /// Returns whether a revision was written.
    #[allow(clippy::too_many_arguments)]
    pub async fn keep_in(
        &self,
        tx: &mut Tx,
        task_id: Id,
        kind: RevisionKind,
        outgoing: Option<&str>,
        outgoing_author: Option<WrapupAuthor>,
        written_at: Option<Instant>,
        trigger: RevisionTrigger,
    ) -> Result<bool> {
        let Some(outgoing) = outgoing.filter(|o| !rekall_common::jstr::is_blank(o)) else {
            return Ok(false);
        };
        let newest = repo::task_revision::find_first_by_task_id_and_kind_order_by_created_at_desc(tx.db(), task_id, kind).await?;
        if newest.as_ref().is_some_and(|n| n.body_markdown == outgoing) {
            return Ok(false);
        }
        let now = self.ctx.now();
        if trigger == RevisionTrigger::HandEdit
            && outgoing_author != Some(WrapupAuthor::Claude)
            && newest.as_ref().is_some_and(|n| n.created_at.is_after(&now.minus(hand_edit_window())))
        {
            return Ok(false);
        }
        let revision = task_revision::Model {
            id: Id::random(),
            task_id,
            kind,
            body_markdown: outgoing.to_string(),
            written_by: outgoing_author,
            written_at,
            created_at: now,
        };
        revision.validate()?;
        revision.into_active_model().insert(tx.db()).await?;
        self.prune(tx, task_id, kind).await?;
        Ok(true)
    }

    pub async fn list(&self, task_id: Id, kind: RevisionKind) -> Result<Vec<TaskRevisionView>> {
        in_read!(&self.ctx, |tx| {
            let all = repo::task_revision::find_by_task_id_and_kind_order_by_created_at_desc(tx.db(), task_id, kind).await?;
            Ok::<_, RekallError>(all.iter().map(TaskRevisionView::of).collect())
        })
    }

    pub async fn find_in(&self, tx: &mut Tx, task_id: Id, revision_id: Id) -> Result<TaskRevisionView> {
        repo::task_revision::find_by_id_and_task_id(tx.db(), revision_id, task_id)
            .await?
            .map(|r| TaskRevisionView::of(&r))
            .ok_or_else(|| RekallError::not_found("Revision", revision_id))
    }

    async fn prune(&self, tx: &Tx, task_id: Id, kind: RevisionKind) -> Result<()> {
        let all = repo::task_revision::find_by_task_id_and_kind_order_by_created_at_desc(tx.db(), task_id, kind).await?;
        if all.len() > KEPT_PER_KIND {
            let stale: Vec<Id> = all[KEPT_PER_KIND..].iter().map(|r| r.id).collect();
            task_revision::Entity::delete_many()
                .filter(task_revision::Column::Id.is_in(stale))
                .exec(tx.db())
                .await?;
        }
        Ok(())
    }
}
