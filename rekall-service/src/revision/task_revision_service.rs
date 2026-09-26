use chrono::TimeDelta;
use rekall_common::{Id, Instant, RekallError, Result};
use rekall_model::task_revision;
use rekall_model::{RevisionKind, WrapupAuthor};
use rekall_repository::repository as repo;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};

use crate::{in_read, Ctx, Tx};

use super::{RevisionTrigger, TaskRevisionView};

pub const KEPT_PER_KIND: usize = 30;

pub fn hand_edit_window() -> TimeDelta {
    TimeDelta::minutes(10)
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
