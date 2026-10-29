use rekall_kernel::{Id, RekallError, Result};
use rekall_store::wrapup::{self, MAX_CHARACTERS};
use rekall_store::{task, WrapupAuthor};
use rekall_store::repository as repo;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use crate::review::TaskReviewService;
use crate::{in_read, in_write, load, text_rule, Ctx, Tx};

use crate::wrapup::{WrapupStreamEvent, WrapupView, Written};

#[derive(Clone)]
pub struct WrapupService {
    ctx: Ctx,
    review: TaskReviewService,
}

impl WrapupService {
    pub fn new(ctx: Ctx, review: TaskReviewService) -> Self {
        Self { ctx, review }
    }

    pub async fn find(&self, task_id: Id) -> Result<Option<WrapupView>> {
        in_read!(&self.ctx, |tx| self.find_in(&tx, task_id).await)
    }

    async fn find_in(&self, tx: &Tx, task_id: Id) -> Result<Option<WrapupView>> {
        match repo::wrapup::find_of_task(tx.db(), task_id).await? {
            Some(found) => Ok(Some(WrapupView::load(tx.db(), &found).await?)),
            None => Ok(None),
        }
    }

    pub async fn find_all(&self) -> Result<Vec<WrapupView>> {
        in_read!(&self.ctx, |tx| {
            let all = repo::wrapup::list_newest_first(tx.db()).await?;
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
            self.write_in(&mut tx, task, body, author).await
        })
    }

    /// The console path: a task named by its id.
    pub async fn write(&self, task_id: Id, body: Option<&str>, author: WrapupAuthor) -> Result<Written> {
        in_write!(&self.ctx, |tx| {
            let task = load::task_or_unknown(tx.db(), task_id).await?;
            self.write_in(&mut tx, task, body, author).await
        })
    }

    async fn write_in(&self, tx: &mut Tx, task: task::Model, body: Option<&str>, author: WrapupAuthor) -> Result<Written> {
        let text = validated(body)?;
        let project = load::project_of(tx.db(), &task).await?;
        let (current, replaced) = match repo::wrapup::find_of_task(tx.db(), task.id).await? {
            None => (self.create(tx, task.id, text, author).await?, None),
            Some(before) => {
                let previous = before.written_by;
                (self.replace(tx, before, text, author).await?, Some(previous))
            }
        };
        let written = Written { wrapup: WrapupView::of(&current, &task, &project), created: replaced.is_none(), replaced };
        // A Claude-authored wrapup advances a stepless task's review line to CLAIMED; a
        // hand-written one does not.
        if author == WrapupAuthor::Claude {
            self.review.claimed_by_wrapup_in(tx, task.id).await?;
        }
        tx.publish(WrapupStreamEvent {
            task_id: task.id,
            wrapup: Some(written.wrapup.clone()),
            deleted: false,
        });
        Ok(written)
    }

    async fn create(&self, tx: &mut Tx, task_id: Id, text: String, author: WrapupAuthor) -> Result<wrapup::Model> {
        let now = self.ctx.now();
        let created = wrapup::Model { id: Id::random(), task_id, body_markdown: text, written_by: author, created_at: now, updated_at: now };
        created.validate()?;
        created.clone().into_active_model().insert(tx.db()).await?;
        Ok(created)
    }

    async fn replace(&self, tx: &mut Tx, before: wrapup::Model, text: String, author: WrapupAuthor) -> Result<wrapup::Model> {
        let mut after = before.clone();
        after.body_markdown = text;
        after.written_by = author;
        repo::update_if_changed(tx.db(), &before, after, self.ctx.now()).await
    }

    pub async fn delete(&self, task_id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            if let Some(found) = repo::wrapup::find_of_task(tx.db(), task_id).await? {
                wrapup::Entity::delete_by_id(found.id).exec(tx.db()).await?;
                tx.publish(WrapupStreamEvent { task_id, wrapup: None, deleted: true });
            }
            Ok::<_, RekallError>(())
        })
    }
}

fn validated(body: Option<&str>) -> Result<String> {
    let body = text_rule::present(body).ok_or_else(|| {
        RekallError::illegal("A wrapup needs a body. To remove one, delete it rather than blanking it.")
    })?;
    text_rule::within(body, MAX_CHARACTERS, |length| {
        format!(
            "A wrapup is capped at {MAX_CHARACTERS} characters and this one is {length}. It describes the state of the \
             implementation, not how it got there; if it does not fit, it is recording the process. Move the detail \
             into a note."
        )
    })
}
