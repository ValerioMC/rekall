//! What a person does to a project's components: list them, add one, edit it, put them in order,
//! delete one. Console-only: a session reads the components through the project's context.

use std::collections::HashSet;

use rekall_common::{jstr, Id, RekallError, Result};
use rekall_model::project_component;
use rekall_repository::repository as repo;
use rekall_service::component::ProjectComponentView;
use rekall_service::{in_read, in_write, Ctx, Tx};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use crate::dto::{ProjectComponentOrderRequest, ProjectComponentRequest};
use crate::extract::uuid;

#[derive(Clone)]
pub struct ProjectComponentCatalogService {
    ctx: Ctx,
}

impl ProjectComponentCatalogService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn list(&self, project_id: Id) -> Result<Vec<ProjectComponentView>> {
        in_read!(&self.ctx, |tx| {
            repo::project::require(tx.db(), project_id).await?;
            let rows = repo::project_component::list_of_project(tx.db(), project_id).await?;
            Ok::<_, RekallError>(rows.iter().map(ProjectComponentView::of).collect())
        })
    }

    pub async fn create(&self, project_id: Id, request: ProjectComponentRequest) -> Result<ProjectComponentView> {
        in_write!(&self.ctx, |tx| {
            repo::project::require(tx.db(), project_id).await?;
            let folder = folder_of(request.folder.as_deref())?;
            let name = request.name.as_deref().filter(|name| !jstr::is_blank(name)).map_or_else(|| folder_name(&folder), |name| jstr::strip(name).to_string());
            require_name_free(&tx, project_id, &name, None).await?;
            let existing = repo::project_component::list_of_project(tx.db(), project_id).await?;
            let now = self.ctx.now();
            let component = project_component::Model {
                id: Id::random(),
                project_id,
                name,
                folder,
                description: request.description.as_deref().map(|text| jstr::strip(text).to_string()).unwrap_or_default(),
                position: existing.iter().map(|component| component.position + 1).max().unwrap_or(0),
                created_at: now,
                updated_at: now,
            };
            component.validate()?;
            component.clone().into_active_model().insert(tx.db()).await?;
            Ok::<_, RekallError>(ProjectComponentView::of(&component))
        })
    }

    pub async fn update(&self, id: Id, request: ProjectComponentRequest) -> Result<ProjectComponentView> {
        in_write!(&self.ctx, |tx| {
            let before = repo::project_component::require(tx.db(), id).await?;
            let mut component = before.clone();
            if let Some(name) = request.name.as_deref() {
                component.name = jstr::strip(name).to_string();
                require_name_free(&tx, component.project_id, &component.name, Some(id)).await?;
            }
            if request.folder.is_some() {
                component.folder = folder_of(request.folder.as_deref())?;
            }
            if let Some(description) = request.description.as_deref() {
                component.description = jstr::strip(description).to_string();
            }
            let component = repo::update_if_changed(tx.db(), &before, component, self.ctx.now()).await?;
            Ok::<_, RekallError>(ProjectComponentView::of(&component))
        })
    }

    /// Positions follow `component_ids`, which has to name every component of the project once.
    pub async fn reorder(&self, project_id: Id, request: ProjectComponentOrderRequest) -> Result<Vec<ProjectComponentView>> {
        in_write!(&self.ctx, |tx| {
            repo::project::require(tx.db(), project_id).await?;
            let components = repo::project_component::list_of_project(tx.db(), project_id).await?;
            let wanted: Vec<Id> = request
                .component_ids
                .clone()
                .unwrap_or_default()
                .iter()
                .map(|raw| uuid("componentIds", raw).map_err(|_| RekallError::illegal(format!("'{raw}' is not a component id."))))
                .collect::<Result<_>>()?;
            let known: HashSet<Id> = components.iter().map(|component| component.id).collect();
            if wanted.len() != known.len() || wanted.iter().copied().collect::<HashSet<_>>() != known {
                return Err(RekallError::illegal("An order has to name every component of the project exactly once."));
            }
            let now = self.ctx.now();
            let mut ordered = Vec::with_capacity(components.len());
            for (position, id) in wanted.iter().enumerate() {
                let mut component = components.iter().find(|component| component.id == *id).cloned().expect("checked against the project's components");
                if component.position != position as i32 {
                    component.position = position as i32;
                    component.updated_at = now;
                    component.clone().into_active_model().reset_all().update(tx.db()).await?;
                }
                ordered.push(ProjectComponentView::of(&component));
            }
            Ok::<_, RekallError>(ordered)
        })
    }

    pub async fn delete(&self, id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let component = repo::project_component::require(tx.db(), id).await?;
            project_component::Entity::delete_by_id(component.id).exec(tx.db()).await?;
            Ok::<_, RekallError>(())
        })
    }
}

/// The folder as stored: stripped, present, and without a trailing separator.
fn folder_of(folder: Option<&str>) -> Result<String> {
    let folder = folder.map(jstr::strip).filter(|folder| !folder.is_empty()).ok_or_else(|| RekallError::illegal("A component needs a folder."))?;
    let trimmed = folder.trim_end_matches(['/', '\\']);
    Ok(if trimmed.is_empty() { folder } else { trimmed }.to_string())
}

/// What the folder is called: its last path segment, the name a component starts with.
fn folder_name(folder: &str) -> String {
    folder.rsplit(['/', '\\']).find(|segment| !segment.is_empty()).unwrap_or(folder).to_string()
}

async fn require_name_free(tx: &Tx, project_id: Id, name: &str, except: Option<Id>) -> Result<()> {
    if jstr::is_blank(name) {
        return Err(RekallError::illegal("A component needs a name."));
    }
    let components = repo::project_component::list_of_project(tx.db(), project_id).await?;
    match components.iter().find(|component| Some(component.id) != except && jstr::equals_ignore_case(&component.name, name)) {
        Some(existing) => Err(RekallError::conflict(format!("This project already has a component named '{}'.", existing.name))),
        None => Ok(()),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/service/project_component_catalog_service_tests.rs"]
mod tests;
