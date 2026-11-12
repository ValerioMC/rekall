//! Notes: written on a task, edited and deleted from the console.

use rekall_kernel::{Id, RekallError, Result};
use rekall_store::{document, DocumentContextMode};
use rekall_store::repository as repo;
use rekall_core::{in_read, in_write, Ctx};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use crate::note::{DocumentCommand, DocumentView};

#[derive(Clone)]
pub struct DocumentService {
    ctx: Ctx,
}

impl DocumentService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn list(&self, task_id: Option<Id>, project_id: Option<Id>) -> Result<Vec<DocumentView>> {
        in_read!(&self.ctx, |tx| {
            let found = if let Some(task_id) = task_id {
                repo::document::notes_of_task_by_title(tx.db(), task_id).await?
            } else if let Some(project_id) = project_id {
                repo::document::list_on_project_tasks(tx.db(), project_id).await?
            } else {
                repo::document::list_newest_first(tx.db()).await?
            };
            Ok(views(&found))
        })
    }

    pub async fn search(&self, term: Option<&str>) -> Result<Vec<DocumentView>> {
        let Some(term) = term.filter(|t| !rekall_kernel::jstr::is_blank(t)) else {
            return Ok(Vec::new());
        };
        in_read!(&self.ctx, |tx| {
            let found = repo::document::search_by_title(tx.db(), rekall_kernel::jstr::trim(term)).await?;
            Ok(views(&found))
        })
    }

    pub async fn create(&self, command: DocumentCommand) -> Result<DocumentView> {
        in_write!(&self.ctx, |tx| {
            let task = repo::task::require(tx.db(), command.task_id).await?;
            let now = self.ctx.now();
            let document = document::Model {
                id: Id::random(),
                task_id: task.id,
                position: repo::document::next_position(tx.db(), task.id).await?,
                title: command.title.clone().unwrap_or_default(),
                kind: command.kind.clone().unwrap_or_default(),
                body_markdown: body(&command),
                source_path: None,
                context_mode: command.context_mode.unwrap_or(DocumentContextMode::Full),
                created_at: now,
                updated_at: now,
            };
            document.validate()?;
            document.clone().into_active_model().insert(tx.db()).await?;
            Ok::<_, RekallError>(DocumentView::of(&document))
        })
    }

    pub async fn update(&self, id: Id, command: DocumentCommand) -> Result<DocumentView> {
        in_write!(&self.ctx, |tx| {
            let before = repo::document::require(tx.db(), id).await?;
            if command.task_id != before.task_id {
                return Err(RekallError::illegal("A note belongs to the task it was written on and cannot move to another."));
            }
            let mut document = before.clone();
            document.title = command.title.clone().unwrap_or_default();
            document.kind = command.kind.clone().unwrap_or_default();
            document.body_markdown = body(&command);
            if let Some(mode) = command.context_mode {
                document.context_mode = mode;
            }
            let document = repo::update_if_changed(tx.db(), &before, document, self.ctx.now()).await?;
            Ok(DocumentView::of(&document))
        })
    }

    pub async fn delete(&self, id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let document = repo::document::require(tx.db(), id).await?;
            document::Entity::delete_by_id(document.id).exec(tx.db()).await?;
            repo::document::close_gaps(tx.db(), document.task_id).await?;
            Ok::<_, RekallError>(())
        })
    }
}

fn body(command: &DocumentCommand) -> String {
    command.body_markdown.clone().unwrap_or_default()
}

fn views(documents: &[document::Model]) -> Vec<DocumentView> {
    documents.iter().map(DocumentView::of).collect()
}
