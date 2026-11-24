//! What a person does to a project's section contexts: add one, rename it, delete it once empty.
//! Console-only, so a session can name a context when it writes a page but never rename or delete one.

use rekall_core::section::rule::context_rules;
use rekall_core::section::{SectionContextView, SectionStreamEvent};
use rekall_core::{in_write, Ctx, Tx};
use rekall_kernel::{Id, RekallError, Result};
use rekall_store::repository as repo;
use rekall_store::section_context;
use sea_orm::EntityTrait;

use crate::section::SectionContextCommand;

#[derive(Clone)]
pub struct SectionContextService {
    ctx: Ctx,
}

impl SectionContextService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn create(&self, project_id: Id, command: SectionContextCommand) -> Result<SectionContextView> {
        in_write!(&self.ctx, |tx| {
            repo::project::require(tx.db(), project_id).await?;
            let title = context_rules::title(command.title.as_deref())?;
            require_title_free(&tx, project_id, &title, None).await?;
            let context = context_rules::add(tx.db(), project_id, title, self.ctx.now()).await?;
            tx.publish(SectionStreamEvent { project_id });
            Ok::<_, RekallError>(SectionContextView::of(&context))
        })
    }

    pub async fn rename(&self, id: Id, command: SectionContextCommand) -> Result<SectionContextView> {
        in_write!(&self.ctx, |tx| {
            let before = repo::section_context::require(tx.db(), id).await?;
            let title = context_rules::title(command.title.as_deref())?;
            require_title_free(&tx, before.project_id, &title, Some(id)).await?;
            let after = section_context::Model { title, ..before.clone() };
            let changed = after != before;
            let saved = repo::update_if_changed(tx.db(), &before, after, self.ctx.now()).await?;
            if changed {
                tx.publish(SectionStreamEvent { project_id: saved.project_id });
            }
            Ok::<_, RekallError>(SectionContextView::of(&saved))
        })
    }

    pub async fn delete(&self, id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let context = repo::section_context::require(tx.db(), id).await?;
            let held = repo::project_section::list_of_context(tx.db(), id).await?;
            if !held.is_empty() {
                return Err(RekallError::conflict(format!(
                    "The context '{}' still holds {} page{}. Move or delete them first.",
                    context.title,
                    held.len(),
                    if held.len() == 1 { "" } else { "s" }
                )));
            }
            section_context::Entity::delete_by_id(id).exec(tx.db()).await?;
            tx.publish(SectionStreamEvent { project_id: context.project_id });
            Ok::<_, RekallError>(())
        })
    }
}

async fn require_title_free(tx: &Tx, project_id: Id, title: &str, except: Option<Id>) -> Result<()> {
    match context_rules::clashing(tx.db(), project_id, title, except).await? {
        Some(existing) => Err(RekallError::conflict(format!("This project already has a context titled '{}'.", existing.title))),
        None => Ok(()),
    }
}
