use rekall_common::{Id, RekallError, Result};
use rekall_model::constraints::Phase;
use rekall_model::project_section;
use rekall_repository::repository as repo;
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use crate::note::OnTitleClash;
use crate::{in_read, in_write, load, Ctx, DomainEvent};

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
            repo::project::find_by_id(tx.db(), project_id).await?.ok_or_else(|| RekallError::not_found("Project", project_id))?;
            let rows = repo::project_section::find_by_project_id_order_by_position_asc(tx.db(), project_id).await?;
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
            let existing = repo::project_section::find_by_project_id_order_by_position_asc(tx.db(), project.id).await?;
            let now = self.ctx.now();
            let project_anchor = project.anchor();

            if let Some(clash) = rules::clashing(tx.db(), project.id, &wanted, None).await? {
                if on_clash == OnTitleClash::Refuse {
                    return Err(rules::clash_refusal(&clash));
                }
                // The title stays as a person typed it; only the body is the session's.
                let mut rewritten = clash.clone();
                rewritten.body_markdown = text;
                if rewritten != clash {
                    rewritten.updated_at = now;
                    rewritten.validate(Phase::Update)?;
                    rewritten.clone().into_active_model().reset_all().update(tx.db()).await?;
                }
                tx.publish(DomainEvent::ProjectSection(SectionStreamEvent { project_id: project.id }));
                return Ok(SectionWritten { title: rewritten.title, project_anchor, sections_on_project: existing.len(), replaced: true });
            }

            let section = project_section::Model {
                id: Id::random(),
                project_id: project.id,
                title: wanted,
                body_markdown: text,
                position: existing.iter().map(|section| section.position + 1).max().unwrap_or(0),
                created_at: now,
                updated_at: now,
            };
            section.validate(Phase::Persist)?;
            section.clone().into_active_model().insert(tx.db()).await?;
            tx.publish(DomainEvent::ProjectSection(SectionStreamEvent { project_id: project.id }));
            Ok(SectionWritten { title: section.title, project_anchor, sections_on_project: existing.len() + 1, replaced: false })
        })
    }
}
