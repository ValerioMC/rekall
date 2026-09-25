//! The one write path for a task's wrapup. Whatever a write replaces, and whatever a delete
//! removes, is handed to the revision service first, so the console's history can bring it back;
//! that includes an edit made by hand, which a session's write would otherwise erase.

use rekall_common::{jstr, Id, Instant, RekallError, Result};
use rekall_model::constraints::Phase;
use rekall_model::wrapup::{self, MAX_CHARACTERS};
use rekall_model::{project, task, RevisionKind, WrapupAuthor};
use rekall_repository::repository as repo;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};
use serde::Serialize;

use crate::review::TaskReviewService;
use crate::revision::{RevisionTrigger, TaskRevisionService};
use crate::{in_read, in_write, load, Ctx, DomainEvent, Tx};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WrapupView {
    pub id: Id,
    pub task_id: Id,
    pub task_label: String,
    pub task_title: String,
    pub project_label: String,
    pub anchor: String,
    pub body_markdown: String,
    pub written_by: WrapupAuthor,
    pub created_at: Instant,
    pub updated_at: Instant,
}

impl WrapupView {
    pub fn of(wrapup: &wrapup::Model, task: &task::Model, project: &project::Model) -> Self {
        Self {
            id: wrapup.id,
            task_id: task.id,
            task_label: task.label.clone(),
            task_title: task.title.clone(),
            project_label: project.label.clone(),
            anchor: format!("project:{} task:{}", project.label, task.label),
            body_markdown: wrapup.body_markdown.clone(),
            written_by: wrapup.written_by,
            created_at: wrapup.created_at,
            updated_at: wrapup.updated_at,
        }
    }

    pub async fn load(db: &impl sea_orm::ConnectionTrait, wrapup: &wrapup::Model) -> Result<Self> {
        let task = load::task_or_unknown(db, wrapup.task_id).await?;
        let project = load::project_of(db, &task).await?;
        Ok(Self::of(wrapup, &task, &project))
    }
}

/// Emitted when a task's wrapup is written or deleted. On a delete `wrapup` is null.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WrapupStreamEvent {
    pub task_id: Id,
    pub wrapup: Option<WrapupView>,
    pub deleted: bool,
}

/// What a write came to: the wrapup, whether it was the first, and whose words it replaced.
#[derive(Clone, Debug)]
pub struct Written {
    pub wrapup: WrapupView,
    pub created: bool,
    pub replaced: Option<WrapupAuthor>,
}

#[derive(Clone)]
pub struct WrapupService {
    ctx: Ctx,
    review: TaskReviewService,
    revisions: TaskRevisionService,
}

impl WrapupService {
    pub fn new(ctx: Ctx, review: TaskReviewService, revisions: TaskRevisionService) -> Self {
        Self { ctx, review, revisions }
    }

    pub async fn find(&self, task_id: Id) -> Result<Option<WrapupView>> {
        in_read!(&self.ctx, |tx| self.find_in(&tx, task_id).await)
    }

    async fn find_in(&self, tx: &Tx, task_id: Id) -> Result<Option<WrapupView>> {
        match repo::wrapup::find_by_task_id(tx.db(), task_id).await? {
            Some(found) => Ok(Some(WrapupView::load(tx.db(), &found).await?)),
            None => Ok(None),
        }
    }

    pub async fn find_all(&self) -> Result<Vec<WrapupView>> {
        in_read!(&self.ctx, |tx| {
            let all = repo::wrapup::find_all_by_order_by_updated_at_desc(tx.db()).await?;
            let mut views = Vec::with_capacity(all.len());
            for found in &all {
                views.push(WrapupView::load(tx.db(), found).await?);
            }
            Ok::<_, RekallError>(views)
        })
    }

    pub async fn find_by_anchor(&self, project_label: Option<&str>, task_label: &str) -> Result<Option<WrapupView>> {
        in_read!(&self.ctx, |tx| {
            let task = load::resolve_task(tx.db(), project_label, task_label).await?;
            self.find_in(&tx, task.id).await
        })
    }

    /// The MCP path: a task named by its anchors.
    pub async fn write_by_anchor(
        &self,
        project_label: Option<&str>,
        task_label: &str,
        body: Option<&str>,
        author: WrapupAuthor,
    ) -> Result<Written> {
        in_write!(&self.ctx, |tx| {
            let task = load::resolve_task(tx.db(), project_label, task_label).await?;
            self.write_in(&mut tx, task, body, author, trigger_of(author)).await
        })
    }

    /// The console path: a task named by its id.
    pub async fn write(&self, task_id: Id, body: Option<&str>, author: WrapupAuthor) -> Result<Written> {
        in_write!(&self.ctx, |tx| {
            let task = load::task_or_unknown(tx.db(), task_id).await?;
            self.write_in(&mut tx, task, body, author, trigger_of(author)).await
        })
    }

    /// Writes an earlier revision back as the current wrapup, keeping the one it replaces.
    pub async fn restore_in(&self, tx: &mut Tx, task_id: Id, body: &str) -> Result<Written> {
        let task = load::task_or_unknown(tx.db(), task_id).await?;
        self.write_in(tx, task, Some(body), WrapupAuthor::Hand, RevisionTrigger::Restore).await
    }

    async fn write_in(
        &self,
        tx: &mut Tx,
        task: task::Model,
        body: Option<&str>,
        author: WrapupAuthor,
        trigger: RevisionTrigger,
    ) -> Result<Written> {
        let text = validated(body)?;
        let project = load::project_of(tx.db(), &task).await?;
        let now = self.ctx.now();
        let written = match repo::wrapup::find_by_task_id(tx.db(), task.id).await? {
            None => {
                let created = wrapup::Model {
                    id: Id::random(),
                    task_id: task.id,
                    body_markdown: text,
                    written_by: author,
                    created_at: now,
                    updated_at: now,
                };
                created.validate(Phase::Persist)?;
                created.clone().into_active_model().insert(tx.db()).await?;
                Written { wrapup: WrapupView::of(&created, &task, &project), created: true, replaced: None }
            }
            Some(before) => {
                let previous = before.written_by;
                if before.body_markdown != text {
                    self.revisions
                        .keep_in(tx, task.id, RevisionKind::Wrapup, Some(&before.body_markdown), Some(previous),
                                 Some(before.updated_at), trigger)
                        .await?;
                }
                let mut after = before.clone();
                after.body_markdown = text;
                after.written_by = author;
                if after != before {
                    after.updated_at = now;
                    after.validate(Phase::Update)?;
                    after.clone().into_active_model().reset_all().update(tx.db()).await?;
                }
                Written { wrapup: WrapupView::of(&after, &task, &project), created: false, replaced: Some(previous) }
            }
        };
        // A Claude-authored wrapup advances a stepless task's review line to CLAIMED; a
        // hand-written one does not.
        if author == WrapupAuthor::Claude {
            self.review.claimed_by_wrapup_in(tx, task.id).await?;
        }
        tx.publish(DomainEvent::Wrapup(WrapupStreamEvent {
            task_id: task.id,
            wrapup: Some(written.wrapup.clone()),
            deleted: false,
        }));
        Ok(written)
    }

    pub async fn delete(&self, task_id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            if let Some(found) = repo::wrapup::find_by_task_id(tx.db(), task_id).await? {
                self.revisions
                    .keep_in(&mut tx, task_id, RevisionKind::Wrapup, Some(&found.body_markdown), Some(found.written_by),
                             Some(found.updated_at), RevisionTrigger::Deletion)
                    .await?;
                wrapup::Entity::delete_by_id(found.id).exec(tx.db()).await?;
                tx.publish(DomainEvent::Wrapup(WrapupStreamEvent { task_id, wrapup: None, deleted: true }));
            }
            Ok::<_, RekallError>(())
        })
    }
}

fn trigger_of(author: WrapupAuthor) -> RevisionTrigger {
    if author == WrapupAuthor::Claude {
        RevisionTrigger::ClaudeWrite
    } else {
        RevisionTrigger::HandEdit
    }
}

fn validated(body: Option<&str>) -> Result<String> {
    let Some(body) = body.filter(|b| !jstr::is_blank(b)) else {
        return Err(RekallError::illegal(
            "A wrapup needs a body. To remove one, delete it rather than blanking it.",
        ));
    };
    let text = jstr::strip(body);
    let length = jstr::len(text);
    if length > MAX_CHARACTERS {
        return Err(RekallError::illegal(format!(
            "A wrapup is capped at {MAX_CHARACTERS} characters and this one is {length}. It describes the state of the \
             implementation, not how it got there; if it does not fit, it is recording the process. Move the detail \
             into a note."
        )));
    }
    Ok(text.to_string())
}
