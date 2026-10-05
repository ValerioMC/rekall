use rekall_common::{Id, RekallError, Result};
use rekall_model::{project, project_section};
use rekall_repository::repository as repo;
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use crate::note::OnTitleClash;
use crate::{in_read, in_write, load, Ctx, DomainEvent, Tx};

use super::{rules, ProjectSectionView, SectionStreamEvent, SectionWritten};

/// Reads a project's sections and writes a session's, starting from the task it is working on.
#[derive(Clone)]
pub struct ProjectSectionService {
    ctx: Ctx,
}

impl ProjectSectionService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn list(&self, project_id: Id) -> Result<Vec<ProjectSectionView>> {
        in_read!(&self.ctx, |tx| {
            repo::project::require(tx.db(), project_id).await?;
            let rows = repo::project_section::list_of_project(tx.db(), project_id).await?;
            Ok::<_, RekallError>(rows.iter().map(ProjectSectionView::of).collect())
        })
    }

    pub async fn write(
        &self,
        project_label: Option<&str>,
        task_label: &str,
        title: Option<&str>,
        body: Option<&str>,
        on_clash: OnTitleClash,
    ) -> Result<SectionWritten> {
        in_write!(&self.ctx, |tx| {
            let task = load::resolve_task(tx.db(), project_label, task_label).await?;
            let project = load::project_of(tx.db(), &task).await?;
            let wanted = rules::title(title)?;
            let text = rules::written_body(body)?;
            let existing = repo::project_section::list_of_project(tx.db(), project.id).await?;

            match rules::clashing(tx.db(), project.id, &wanted, None).await? {
                Some(clash) if on_clash == OnTitleClash::Refuse => Err(rules::clash_refusal(&clash)),
                Some(clash) => self.rewrite(&mut tx, &project, &existing, clash, text).await,
                None => self.add(&mut tx, &project, &existing, wanted, text).await,
            }
        })
    }

    /// The title stays as a person typed it; only the body is the session's.
    async fn rewrite(
        &self,
        tx: &mut Tx,
        project: &project::Model,
        existing: &[project_section::Model],
        clash: project_section::Model,
        text: String,
    ) -> Result<SectionWritten> {
        let mut rewritten = clash.clone();
        rewritten.body_markdown = text;
        let rewritten = repo::update_if_changed(tx.db(), &clash, rewritten, self.ctx.now()).await?;
        tx.publish(DomainEvent::ProjectSection(SectionStreamEvent { project_id: project.id }));
        Ok(SectionWritten { title: rewritten.title, project_anchor: project.anchor(), sections_on_project: existing.len(), replaced: true })
    }

    /// A new section goes last.
    async fn add(
        &self,
        tx: &mut Tx,
        project: &project::Model,
        existing: &[project_section::Model],
        title: String,
        text: String,
    ) -> Result<SectionWritten> {
        let now = self.ctx.now();
        let section = project_section::Model {
            id: Id::random(),
            project_id: project.id,
            title,
            body_markdown: text,
            position: existing.iter().map(|section| section.position + 1).max().unwrap_or(0),
            created_at: now,
            updated_at: now,
        };
        section.validate()?;
        section.clone().into_active_model().insert(tx.db()).await?;
        tx.publish(DomainEvent::ProjectSection(SectionStreamEvent { project_id: project.id }));
        Ok(SectionWritten { title: section.title, project_anchor: project.anchor(), sections_on_project: existing.len() + 1, replaced: false })
    }
}
