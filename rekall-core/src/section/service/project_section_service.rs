use rekall_kernel::{jstr, Id, RekallError, Result};
use rekall_store::{project, project_section};
use rekall_store::repository as repo;
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use crate::note::OnTitleClash;
use crate::{in_read, in_write, load, text_rule, Ctx, Tx};

use crate::section::{rule::rules, ProjectSectionView, SectionStreamEvent, SectionWritten, SectionsRead};

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

    /// A session reads the sections of the project it names, or of a task's project when a task
    /// label comes. With a title, only that section (ignoring case); one the project lacks is
    /// refused with the titles it has.
    pub async fn read(
        &self,
        project_label: Option<&str>,
        task_label: Option<&str>,
        title: Option<&str>,
    ) -> Result<SectionsRead> {
        in_read!(&self.ctx, |tx| {
            let project = match (task_label, project_label) {
                (Some(task_label), _) => {
                    let task = load::resolve_task(tx.db(), project_label, task_label).await?;
                    load::project_of(tx.db(), &task).await?
                }
                (None, Some(project_label)) => load::resolve_project(tx.db(), project_label).await?,
                (None, None) => return Err(RekallError::illegal("Name a project to read the sections of.")),
            };
            let rows = repo::project_section::list_of_project(tx.db(), project.id).await?;
            let Some(wanted) = text_rule::present(title) else {
                let sections = rows.iter().map(ProjectSectionView::of).collect();
                return Ok(SectionsRead { project_anchor: project.anchor(), sections, asked_for_one: false });
            };
            let found = rows.iter().find(|section| jstr::equals_ignore_case(jstr::strip(&section.title), wanted));
            match found {
                Some(section) => Ok(SectionsRead {
                    project_anchor: project.anchor(),
                    sections: vec![ProjectSectionView::of(section)],
                    asked_for_one: true,
                }),
                None => Err(no_such_section(&project, wanted, &rows)),
            }
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
        tx.publish(SectionStreamEvent { project_id: project.id });
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
        tx.publish(SectionStreamEvent { project_id: project.id });
        Ok(SectionWritten { title: section.title, project_anchor: project.anchor(), sections_on_project: existing.len() + 1, replaced: false })
    }
}

fn no_such_section(project: &project::Model, wanted: &str, rows: &[project_section::Model]) -> RekallError {
    let titles: Vec<&str> = rows.iter().map(|section| section.title.as_str()).collect();
    let held = if titles.is_empty() { "has no sections".to_string() } else { format!("has: {}", titles.join(", ")) };
    RekallError::illegal(format!("`{}` has no section titled '{wanted}'. It {held}.", project.anchor()))
}
