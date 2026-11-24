use rekall_kernel::{jstr, Id, RekallError, Result};
use rekall_store::{project, project_section, section_context};
use rekall_store::repository as repo;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use crate::note::OnTitleClash;
use crate::{in_read, in_write, load, text_rule, Ctx, ProjectRef, Tx};

use crate::section::rule::placement_rules::{self, Placement};
use crate::section::rule::{context_rules, reorganization_rules, rules};
use crate::section::{PageGroup, ProjectSectionView, SectionContextView, SectionStreamEvent, SectionWritten, SectionsRead, SectionsReorganized};

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

    pub async fn contexts(&self, project_id: Id) -> Result<Vec<SectionContextView>> {
        in_read!(&self.ctx, |tx| {
            repo::project::require(tx.db(), project_id).await?;
            let rows = repo::section_context::list_of_project(tx.db(), project_id).await?;
            Ok::<_, RekallError>(rows.iter().map(SectionContextView::of).collect())
        })
    }

    /// A session reads the sections of the project it names, or of a task's project when a task
    /// label comes. With a title, only that section (ignoring case); one the project lacks is
    /// refused with the titles it has.
    pub async fn read(
        &self,
        project_ref: Option<&ProjectRef>,
        task_label: Option<&str>,
        title: Option<&str>,
    ) -> Result<SectionsRead> {
        in_read!(&self.ctx, |tx| {
            let project = project_named(tx.db(), project_ref, task_label).await?;
            let rows = repo::project_section::list_of_project(tx.db(), project.id).await?;
            let contexts: Vec<SectionContextView> =
                repo::section_context::list_of_project(tx.db(), project.id).await?.iter().map(SectionContextView::of).collect();
            let Some(wanted) = text_rule::present(title) else {
                let sections = rows.iter().map(ProjectSectionView::of).collect();
                return Ok(SectionsRead { project_anchor: project.anchor(), contexts, sections, asked_for_one: false });
            };
            let found = rows.iter().find(|section| jstr::equals_ignore_case(jstr::strip(&section.title), wanted));
            match found {
                Some(section) => Ok(SectionsRead {
                    project_anchor: project.anchor(),
                    contexts,
                    sections: vec![ProjectSectionView::of(section)],
                    asked_for_one: true,
                }),
                None => Err(no_such_section(&project, wanted, &rows)),
            }
        })
    }

    /// A session regroups the project's pages: `groups` names every page once, under the contexts
    /// it wants. Pages move, contexts it names are added and the ones left empty are dropped; no
    /// page is renamed, rewritten or deleted.
    pub async fn reorganize(
        &self,
        project_ref: Option<&ProjectRef>,
        task_label: Option<&str>,
        groups: &[PageGroup],
    ) -> Result<SectionsReorganized> {
        in_write!(&self.ctx, |tx| {
            let project = project_named(tx.db(), project_ref, task_label).await?;
            let sections = repo::project_section::list_of_project(tx.db(), project.id).await?;
            let wanted = reorganization_rules::checked(groups, &sections)?;
            let now = self.ctx.now();
            let mut contexts = repo::section_context::list_of_project(tx.db(), project.id).await?;
            let mut placements = Vec::with_capacity(wanted.len());
            for group in &wanted {
                let context = match contexts.iter().find(|known| jstr::equals_ignore_case(jstr::strip(&known.title), &group.context)) {
                    Some(known) => known.clone(),
                    None => {
                        let added = context_rules::add(tx.db(), project.id, group.context.clone(), now).await?;
                        contexts.push(added.clone());
                        added
                    }
                };
                placements.push(Placement { context_id: context.id, section_ids: group.page_ids.clone() });
            }
            let pages_moved = placement_rules::apply(tx.db(), &contexts, &sections, &placements, now).await?;
            let contexts_dropped = self.drop_unnamed(&mut tx, &contexts, &placements).await?;
            tx.publish(SectionStreamEvent { project_id: project.id });
            Ok::<_, RekallError>(SectionsReorganized {
                project_anchor: project.anchor(),
                contexts: wanted.iter().map(|group| (group.context.clone(), group.page_ids.len())).collect(),
                pages_moved,
                contexts_dropped,
            })
        })
    }

    /// Every page was placed, so a context the session did not name is empty and goes.
    async fn drop_unnamed(&self, tx: &mut Tx, contexts: &[section_context::Model], placements: &[Placement]) -> Result<usize> {
        let unnamed: Vec<&section_context::Model> =
            contexts.iter().filter(|context| !placements.iter().any(|placement| placement.context_id == context.id)).collect();
        for context in &unnamed {
            section_context::Entity::delete_by_id(context.id).exec(tx.db()).await?;
        }
        Ok(unnamed.len())
    }

    pub async fn write(
        &self,
        project_label: Option<&str>,
        task_label: &str,
        title: Option<&str>,
        body: Option<&str>,
        context: Option<&str>,
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
                None => self.add(&mut tx, &project, &existing, wanted, text, context).await,
            }
        })
    }

    /// The title stays as a person typed it, and so does the context; only the body is the session's.
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
        let held_in = repo::section_context::require(tx.db(), rewritten.context_id).await?;
        tx.publish(SectionStreamEvent { project_id: project.id });
        Ok(SectionWritten {
            title: rewritten.title,
            context_title: held_in.title,
            project_anchor: project.anchor(),
            sections_on_project: existing.len(),
            replaced: true,
        })
    }

    /// A new section goes last in its context.
    async fn add(
        &self,
        tx: &mut Tx,
        project: &project::Model,
        existing: &[project_section::Model],
        title: String,
        text: String,
        context: Option<&str>,
    ) -> Result<SectionWritten> {
        let now = self.ctx.now();
        let landing: section_context::Model = context_rules::landing(tx.db(), project.id, context, now).await?;
        let section = project_section::Model {
            id: Id::random(),
            project_id: project.id,
            context_id: landing.id,
            title,
            body_markdown: text,
            position: context_rules::next_section_position(tx.db(), landing.id).await?,
            created_at: now,
            updated_at: now,
        };
        section.validate()?;
        section.clone().into_active_model().insert(tx.db()).await?;
        tx.publish(SectionStreamEvent { project_id: project.id });
        Ok(SectionWritten {
            title: section.title,
            context_title: landing.title,
            project_anchor: project.anchor(),
            sections_on_project: existing.len() + 1,
            replaced: false,
        })
    }
}

fn no_such_section(project: &project::Model, wanted: &str, rows: &[project_section::Model]) -> RekallError {
    let titles: Vec<&str> = rows.iter().map(|section| section.title.as_str()).collect();
    let held = if titles.is_empty() { "has no sections".to_string() } else { format!("has: {}", titles.join(", ")) };
    RekallError::illegal(format!("`{}` has no section titled '{wanted}'. It {held}.", project.anchor()))
}

async fn project_named(db: &impl sea_orm::ConnectionTrait, project_ref: Option<&ProjectRef>, task_label: Option<&str>) -> Result<project::Model> {
    match (task_label, project_ref) {
        (Some(task_label), _) => {
            let task = load::resolve_task(db, project_ref.map(|reference| reference.label.as_str()), task_label).await?;
            load::project_of(db, &task).await
        }
        (None, Some(reference)) => load::resolve_project(db, reference).await,
        (None, None) => Err(RekallError::illegal("Name a project, or a task and its project.")),
    }
}
