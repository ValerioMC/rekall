//! What a person does to a project's sections: write one, edit it, put the sections in order,
//! delete one. Console-only, so a session can add a section but never rename, reorder or remove.

use std::collections::HashSet;

use rekall_core::Ctx;

use rekall_kernel::{Id, RekallError, Result};
use rekall_store::project_section;
use rekall_store::repository as repo;
use rekall_core::section::{rules, ProjectSectionView, SectionStreamEvent};
use rekall_core::{in_write, Tx};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use crate::section::{ProjectSectionOrderCommand, ProjectSectionCommand};

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
            let existing = repo::project_section::list_of_project(tx.db(), project_id).await?;
            let now = self.ctx.now();
            let section = project_section::Model {
                id: Id::random(),
                project_id,
                title,
                body_markdown: command.body_markdown.clone().unwrap_or_default(),
                position: existing.iter().map(|section| section.position + 1).max().unwrap_or(0),
                created_at: now,
                updated_at: now,
            };
            section.validate()?;
            section.clone().into_active_model().insert(tx.db()).await?;
            tx.publish(SectionStreamEvent { project_id });
            Ok::<_, RekallError>(ProjectSectionView::of(&section))
        })
    }

    pub async fn update(&self, id: Id, command: ProjectSectionCommand) -> Result<ProjectSectionView> {
        in_write!(&self.ctx, |tx| {
            let before = require(&tx, id).await?;
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

    /// Positions follow `section_ids`, which has to name every section of the project once.
    pub async fn reorder(&self, project_id: Id, command: ProjectSectionOrderCommand) -> Result<Vec<ProjectSectionView>> {
        in_write!(&self.ctx, |tx| {
            repo::project::require(tx.db(), project_id).await?;
            let sections = repo::project_section::list_of_project(tx.db(), project_id).await?;
            let wanted: Vec<Id> = command
                .section_ids
                .clone()
                .unwrap_or_default()
                .iter()
                .map(|raw| raw.parse::<Id>().map_err(|_| RekallError::illegal(format!("'{raw}' is not a section id."))))
                .collect::<Result<_>>()?;
            let known: HashSet<Id> = sections.iter().map(|section| section.id).collect();
            if wanted.len() != known.len() || wanted.iter().collect::<HashSet<_>>() != known.iter().collect::<HashSet<_>>() {
                return Err(RekallError::illegal("An order has to name every section of the project exactly once."));
            }
            let now = self.ctx.now();
            let mut ordered = Vec::with_capacity(sections.len());
            for (position, id) in wanted.iter().enumerate() {
                let mut section = sections.iter().find(|section| section.id == *id).cloned().expect("checked against the project's sections");
                if section.position != position as i32 {
                    section.position = position as i32;
                    section.updated_at = now;
                    section.clone().into_active_model().reset_all().update(tx.db()).await?;
                }
                ordered.push(ProjectSectionView::of(&section));
            }
            tx.publish(SectionStreamEvent { project_id });
            Ok::<_, RekallError>(ordered)
        })
    }

    pub async fn delete(&self, id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let section = require(&tx, id).await?;
            project_section::Entity::delete_by_id(section.id).exec(tx.db()).await?;
            tx.publish(SectionStreamEvent { project_id: section.project_id });
            Ok::<_, RekallError>(())
        })
    }
}

async fn require(tx: &Tx, id: Id) -> Result<project_section::Model> {
    repo::project_section::find_by_id(tx.db(), id).await?.ok_or_else(|| RekallError::not_found("Section", id))
}

async fn require_title_free(tx: &Tx, project_id: Id, title: &str, except: Option<Id>) -> Result<()> {
    match rules::clashing(tx.db(), project_id, title, except).await? {
        Some(existing) => Err(RekallError::conflict(format!("This project already has a section titled '{}'.", existing.title))),
        None => Ok(()),
    }
}
