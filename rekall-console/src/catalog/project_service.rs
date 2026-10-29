//! Projects: listed, created, edited and deleted from the console, and the git status of the
//! folder a project points at. Deleting one cascades to its tasks.

use rekall_common::{jstr, Id, RekallError, Result};
use rekall_store::{company, project, ProjectStatus, Slug};
use rekall_store::repository as repo;
use rekall_service::commit::GitRepositoryInspector;
use rekall_service::{in_read, in_write, Ctx, Tx};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use crate::catalog::Snapshot;
use crate::catalog::{ProjectRepositoryView, ProjectCommand, ProjectView};

const DEFAULT_ICON: &str = "folder";

#[derive(Clone)]
pub struct ProjectService {
    ctx: Ctx,
    repositories: GitRepositoryInspector,
}

impl ProjectService {
    pub fn new(ctx: Ctx, repositories: GitRepositoryInspector) -> Self {
        Self { ctx, repositories }
    }

    pub async fn list(&self) -> Result<Vec<ProjectView>> {
        in_read!(&self.ctx, |tx| {
            let snapshot = Snapshot::of_catalog(tx.db()).await?;
            let projects = repo::project::list_in_catalog_order(tx.db()).await?;
            projects.iter().map(|project| snapshot.project_response(project)).collect::<Result<Vec<_>>>()
        })
    }

    pub async fn get(&self, id: Id) -> Result<ProjectView> {
        in_read!(&self.ctx, |tx| {
            let project = repo::project::require(tx.db(), id).await?;
            Snapshot::for_project(tx.db(), &project).await?.project_response(&project)
        })
    }

    pub async fn create(&self, command: ProjectCommand) -> Result<ProjectView> {
        in_write!(&self.ctx, |tx| {
            let label = Slug::of(command.label.as_deref())?;
            let company = require_owning_company(&tx, command.company_id).await?;
            let now = self.ctx.now();
            let mut project = project::Model {
                id: Id::random(),
                label,
                title: trimmed_title(&command),
                status: ProjectStatus::Active,
                icon: DEFAULT_ICON.to_string(),
                description: None,
                blueprint_markdown: None,
                repo_folder: None,
                auto_commit: false,
                company_id: company.id,
                created_at: now,
                updated_at: now,
            };
            self.apply_request(&mut project, &command).await;
            project.validate()?;
            require_label_free(&tx, &project, None).await?;
            project.clone().into_active_model().insert(tx.db()).await?;
            Snapshot::for_project(tx.db(), &project).await?.project_response(&project)
        })
    }

    pub async fn update(&self, id: Id, command: ProjectCommand) -> Result<ProjectView> {
        in_write!(&self.ctx, |tx| {
            let before = repo::project::require(tx.db(), id).await?;
            let mut project = before.clone();
            project.label = Slug::of(command.label.as_deref())?;
            project.title = trimmed_title(&command);
            project.company_id = require_owning_company(&tx, command.company_id).await?.id;
            self.apply_request(&mut project, &command).await;
            require_label_free(&tx, &project, Some(id)).await?;
            let project = repo::update_if_changed(tx.db(), &before, project, self.ctx.now()).await?;
            Snapshot::for_project(tx.db(), &project).await?.project_response(&project)
        })
    }

    pub async fn delete(&self, id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let project = repo::project::require(tx.db(), id).await?;
            project::Entity::delete_by_id(project.id).exec(tx.db()).await?;
            Ok::<_, RekallError>(())
        })
    }

    /// The folder as git sees it, plus whether the project auto-commits there.
    pub async fn repository(&self, id: Id) -> Result<ProjectRepositoryView> {
        in_read!(&self.ctx, |tx| {
            let project = repo::project::require(tx.db(), id).await?;
            let status = self.repositories.inspect(project.repo_folder.as_deref()).await;
            Ok::<_, RekallError>(ProjectRepositoryView::of(status, project.auto_commit))
        })
    }

    /// The fields a request sets on create and update alike; none of them can be refused.
    async fn apply_request(&self, project: &mut project::Model, command: &ProjectCommand) {
        project.description = command.description.clone();
        project.blueprint_markdown = command.blueprint_markdown.clone();
        project.repo_folder = command.repo_folder.as_deref().filter(|folder| !jstr::is_blank(folder)).map(|folder| jstr::trim(folder).to_string());
        project.auto_commit = self.auto_commit_allowed(command.auto_commit, project.repo_folder.as_deref()).await;
        project.status = command.status.unwrap_or(ProjectStatus::Active);
        project.icon = match command.icon.as_deref() {
            Some(icon) if !jstr::is_blank(icon) => jstr::trim(icon).to_string(),
            _ => DEFAULT_ICON.to_string(),
        };
    }

    /// Auto-commit only means something in a git repository: asked for anywhere else, it stays off.
    async fn auto_commit_allowed(&self, requested: Option<bool>, repo_folder: Option<&str>) -> bool {
        requested == Some(true) && self.repositories.is_repository(repo_folder).await
    }
}

fn trimmed_title(command: &ProjectCommand) -> String {
    jstr::trim(command.title.as_deref().unwrap_or_default()).to_string()
}

async fn require_owning_company(tx: &Tx, id: Option<Id>) -> Result<company::Model> {
    let Some(id) = id else {
        return Err(RekallError::not_found_msg("A project must belong to a company"));
    };
    repo::company::require(tx.db(), id).await
}

/// A label names one project within its company, or a `project:` anchor would be ambiguous.
async fn require_label_free(tx: &Tx, project: &project::Model, except: Option<Id>) -> Result<()> {
    if repo::project::label_taken(tx.db(), project.company_id, &project.label, except).await? {
        return Err(RekallError::conflict(
            "Another project in this company already uses that label. An anchor has to name one record.",
        ));
    }
    Ok(())
}
