//! What a person does to a project's sections: write one, edit it, delete one. Console-only, so a
//! session can add a section but never rename, move or remove one. Where they sit is
//! `SectionLayoutService`'s.

use rekall_core::Ctx;

use rekall_kernel::{Id, RekallError, Result};
use rekall_store::{project_section, section_context};
use rekall_store::repository as repo;
use rekall_core::section::rule::{context_rules, rules};
use rekall_core::section::{ProjectSectionView, SectionStreamEvent};
use rekall_core::{in_write, Tx};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use crate::section::{ProjectSectionCommand, SectionWriteOutcome};

#[derive(Clone)]
pub struct ProjectSectionCatalogService {
    ctx: Ctx,
}

impl ProjectSectionCatalogService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn create(&self, project_id: Id, command: ProjectSectionCommand) -> Result<ProjectSectionView> {
        in_write!(&self.ctx, |tx| {
            repo::project::require(tx.db(), project_id).await?;
            let title = rules::title(command.title.as_deref())?;
            require_title_free(&tx, project_id, &title, None).await?;
            let context = self.context_for_new_section(&tx, project_id, command.context_id.as_deref()).await?;
            let section = self.insert_in(&mut tx, &context, title, command.body_markdown.clone().unwrap_or_default()).await?;
            Ok::<_, RekallError>(ProjectSectionView::of(&section))
        })
    }

    async fn context_for_new_section(&self, tx: &Tx, project_id: Id, context_id: Option<&str>) -> Result<section_context::Model> {
        let Some(raw) = context_id else {
            return context_rules::landing(tx.db(), project_id, None, self.ctx.now()).await;
        };
        let id = raw.parse::<Id>().map_err(|_| RekallError::illegal(format!("'{raw}' is not a context id.")))?;
        let context = repo::section_context::require(tx.db(), id).await?;
        if context.project_id != project_id {
            return Err(RekallError::illegal("That context belongs to another project."));
        }
        Ok(context)
    }

    /// Writes an imported section: a new title is added, one the project has is rewritten only when
    /// `replace` and otherwise kept as a person left it.
    pub async fn import_in(&self, tx: &mut Tx, project_id: Id, title: &str, body: String, replace: bool) -> Result<SectionWriteOutcome> {
        let title = rules::title(Some(title))?;
        let Some(existing) = rules::clashing(tx.db(), project_id, &title, None).await? else {
            let context = context_rules::landing(tx.db(), project_id, None, self.ctx.now()).await?;
            self.insert_in(tx, &context, title, body).await?;
            return Ok(SectionWriteOutcome::Created);
        };
        if !replace {
            return Ok(SectionWriteOutcome::Kept);
        }
        let rewritten = project_section::Model { body_markdown: body, ..existing.clone() };
        if rewritten != existing {
            repo::update_if_changed(tx.db(), &existing, rewritten, self.ctx.now()).await?;
            tx.publish(SectionStreamEvent { project_id });
        }
        Ok(SectionWriteOutcome::Replaced)
    }

    /// A new section goes last in its context.
    async fn insert_in(&self, tx: &mut Tx, context: &section_context::Model, title: String, body: String) -> Result<project_section::Model> {
        let now = self.ctx.now();
        let project_id = context.project_id;
        let section = project_section::Model {
            id: Id::random(),
            project_id,
            context_id: context.id,
            title,
            body_markdown: body,
            position: context_rules::next_section_position(tx.db(), context.id).await?,
            created_at: now,
            updated_at: now,
        };
        section.validate()?;
        section.clone().into_active_model().insert(tx.db()).await?;
        tx.publish(SectionStreamEvent { project_id });
        Ok(section)
    }

    pub async fn update(&self, id: Id, command: ProjectSectionCommand) -> Result<ProjectSectionView> {
        in_write!(&self.ctx, |tx| {
            let before = repo::project_section::require(tx.db(), id).await?;
            let mut section = before.clone();
            if command.title.is_some() {
                section.title = rules::title(command.title.as_deref())?;
                require_title_free(&tx, section.project_id, &section.title, Some(id)).await?;
            }
            if let Some(body) = &command.body_markdown {
                section.body_markdown = body.clone();
            }
            if section != before {
                tx.publish(SectionStreamEvent { project_id: section.project_id });
            }
            let section = repo::update_if_changed(tx.db(), &before, section, self.ctx.now()).await?;
            Ok::<_, RekallError>(ProjectSectionView::of(&section))
        })
    }

    pub async fn delete(&self, id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let section = repo::project_section::require(tx.db(), id).await?;
            project_section::Entity::delete_by_id(section.id).exec(tx.db()).await?;
            tx.publish(SectionStreamEvent { project_id: section.project_id });
            Ok::<_, RekallError>(())
        })
    }
}

async fn require_title_free(tx: &Tx, project_id: Id, title: &str, except: Option<Id>) -> Result<()> {
    match rules::clashing(tx.db(), project_id, title, except).await? {
        Some(existing) => Err(RekallError::conflict(format!("This project already has a section titled '{}'.", existing.title))),
        None => Ok(()),
    }
}
