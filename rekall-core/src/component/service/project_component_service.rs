use std::path::Path;

use rekall_kernel::{Id, RekallError, Result};
use rekall_store::project_component;
use rekall_store::repository as repo;
use rekall_store::project;
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use crate::component::rule::rules;
use crate::component::{ComponentStreamEvent, ComponentWritten, ComponentsRead, ProjectComponentView};
use crate::{in_read, in_write, load, text_rule, Ctx, ProjectRef, Tx};

/// Reads a project's components whole and writes a session's, found by name or by folder.
#[derive(Clone)]
pub struct ProjectComponentService {
    ctx: Ctx,
}

/// A component's fields after the rules have passed them.
struct Validated {
    name: String,
    folder: String,
    summary: String,
    description: String,
}

pub struct ComponentDraft<'a> {
    pub name: Option<&'a str>,
    pub folder: Option<&'a str>,
    pub summary: Option<&'a str>,
    pub description: Option<&'a str>,
}

impl ProjectComponentService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    /// Without a name, every component of the project; with one, that component (ignoring case),
    /// or a refusal listing the names the project has.
    pub async fn read(&self, project_ref: &ProjectRef, name: Option<&str>) -> Result<ComponentsRead> {
        in_read!(&self.ctx, |tx| {
            let project = load::resolve_project(tx.db(), project_ref).await?;
            let rows = repo::project_component::list_of_project(tx.db(), project.id).await?;
            let Some(wanted) = text_rule::present(name) else {
                let components = rows.iter().map(ProjectComponentView::of).collect();
                return Ok(ComponentsRead { project_anchor: project.anchor(), components, asked_for_one: false });
            };
            match rows.iter().find(|component| rules::is_same_name(&component.name, wanted)) {
                Some(component) => Ok(ComponentsRead { project_anchor: project.anchor(), components: vec![ProjectComponentView::of(component)], asked_for_one: true }),
                None => Err(no_such_component(&project, wanted, &rows)),
            }
        })
    }

    /// The component named like `draft.name` or holding `draft.folder` is rewritten and keeps its
    /// name and place; otherwise a new one goes last. A relative folder is taken from the project's.
    pub async fn write(&self, project_ref: &ProjectRef, draft: ComponentDraft<'_>) -> Result<ComponentWritten> {
        in_write!(&self.ctx, |tx| {
            let project = load::resolve_project(tx.db(), project_ref).await?;
            let written = Validated {
                name: rules::name(draft.name)?,
                folder: absolute_folder(&project, &rules::folder(draft.folder)?),
                summary: rules::written_summary(draft.summary)?,
                description: rules::written_description(draft.description)?,
            };
            let existing = repo::project_component::list_of_project(tx.db(), project.id).await?;
            let known = existing
                .iter()
                .find(|component| rules::is_same_name(&component.name, &written.name))
                .or_else(|| existing.iter().find(|component| component.folder == written.folder));
            match known {
                Some(known) => self.rewrite(&mut tx, &project, existing.len(), known.clone(), written).await,
                None => self.add(&mut tx, &project, &existing, written).await,
            }
        })
    }

    async fn rewrite(
        &self,
        tx: &mut Tx,
        project: &project::Model,
        components_on_project: usize,
        known: project_component::Model,
        written: Validated,
    ) -> Result<ComponentWritten> {
        let mut rewritten = known.clone();
        rewritten.folder = written.folder;
        rewritten.summary = written.summary;
        rewritten.description = written.description;
        let rewritten = repo::update_if_changed(tx.db(), &known, rewritten, self.ctx.now()).await?;
        tx.publish(ComponentStreamEvent { project_id: project.id });
        Ok(ComponentWritten { name: rewritten.name, project_anchor: project.anchor(), components_on_project, replaced: true })
    }

    async fn add(
        &self,
        tx: &mut Tx,
        project: &project::Model,
        existing: &[project_component::Model],
        written: Validated,
    ) -> Result<ComponentWritten> {
        let now = self.ctx.now();
        let component = project_component::Model {
            id: Id::random(),
            project_id: project.id,
            name: written.name,
            folder: written.folder,
            summary: written.summary,
            description: written.description,
            position: existing.iter().map(|component| component.position + 1).max().unwrap_or(0),
            created_at: now,
            updated_at: now,
        };
        component.validate()?;
        component.clone().into_active_model().insert(tx.db()).await?;
        tx.publish(ComponentStreamEvent { project_id: project.id });
        Ok(ComponentWritten { name: component.name, project_anchor: project.anchor(), components_on_project: existing.len() + 1, replaced: false })
    }
}

fn absolute_folder(project: &project::Model, folder: &str) -> String {
    match project.repo_folder.as_deref() {
        Some(root) if !Path::new(folder).is_absolute() => Path::new(root).join(folder).to_string_lossy().into_owned(),
        _ => folder.to_string(),
    }
}

fn no_such_component(project: &project::Model, wanted: &str, rows: &[project_component::Model]) -> RekallError {
    let names: Vec<&str> = rows.iter().map(|component| component.name.as_str()).collect();
    let held = if names.is_empty() { "has no components".to_string() } else { format!("has: {}", names.join(", ")) };
    RekallError::illegal(format!("`{}` has no component named '{wanted}'. It {held}.", project.anchor()))
}
