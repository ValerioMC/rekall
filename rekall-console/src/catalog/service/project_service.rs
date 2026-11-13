//! Projects: listed, created, edited and deleted from the console, and the git status of the
//! folder a project points at. Deleting one cascades to its tasks.

use rekall_kernel::{jstr, Id, RekallError, Result};
use rekall_store::{company, project, ProjectStatus, Slug, TaskKey};
use rekall_store::repository as repo;
use rekall_core::commit::GitRepositoryInspector;
use rekall_core::worktree::{PullRequestFlowService, PullRequestFlowSite};
use rekall_core::{in_read, in_write, Ctx, Tx};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use crate::catalog::Snapshot;
use crate::catalog::{ProjectRepositoryView, ProjectCommand, ProjectView};

const DEFAULT_ICON: &str = "folder";

#[derive(Clone)]
pub struct ProjectService {
    ctx: Ctx,
    repositories: GitRepositoryInspector,
    pull_request_flow: PullRequestFlowService,
}

impl ProjectService {
    pub fn new(ctx: Ctx, repositories: GitRepositoryInspector, pull_request_flow: PullRequestFlowService) -> Self {
        Self { ctx, repositories, pull_request_flow }
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
            let task_key = chosen_task_key(&tx, command.task_key.as_deref(), &label).await?;
            let mut project = project::Model {
                id: Id::random(),
                label,
                task_key,
                title: trimmed_title(&command),
                status: ProjectStatus::Active,
                icon: DEFAULT_ICON.to_string(),
                description: None,
                blueprint_markdown: None,
                repo_folder: None,
                auto_commit: false,
                main_branch: None,
                pull_request_flow: false,
                company_id: company.id,
                created_at: now,
                updated_at: now,
            };
            self.apply_request(&tx, &mut project, &command).await?;
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
            self.apply_request(&tx, &mut project, &command).await?;
            require_label_free(&tx, &project, Some(id)).await?;
            self.change_task_key(&tx, &mut project, command.task_key.as_deref()).await?;
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

    /// The fields a request sets on create and update alike.
    async fn apply_request(&self, tx: &Tx, project: &mut project::Model, command: &ProjectCommand) -> Result<()> {
        project.description = command.description.clone();
        project.blueprint_markdown = command.blueprint_markdown.clone();
        project.repo_folder = command.repo_folder.as_deref().filter(|folder| !jstr::is_blank(folder)).map(|folder| jstr::trim(folder).to_string());
        project.auto_commit = self.auto_commit_allowed(command.auto_commit, project.repo_folder.as_deref()).await;
        project.main_branch = main_branch_of(command.main_branch.as_deref())?;
        project.pull_request_flow = self.pull_request_flow_of(tx, project, command.pull_request_flow).await?;
        project.status = command.status.unwrap_or(ProjectStatus::Active);
        project.icon = match command.icon.as_deref() {
            Some(icon) if !jstr::is_blank(icon) => jstr::trim(icon).to_string(),
            _ => DEFAULT_ICON.to_string(),
        };
        Ok(())
    }

    /// Not asked for keeps the flow as it is; asked for on is refused unless the project can run it.
    async fn pull_request_flow_of(&self, tx: &Tx, project: &project::Model, requested: Option<bool>) -> Result<bool> {
        let Some(requested) = requested else { return Ok(project.pull_request_flow) };
        if requested {
            self.pull_request_flow.require_ground_in(tx, project.id, self.flow_site_of(project).await).await?;
        }
        Ok(requested)
    }

    async fn flow_site_of(&self, project: &project::Model) -> PullRequestFlowSite {
        let has_folder = project.repo_folder.is_some();
        let has_branch = project.main_branch.is_some() || self.repositories.inspect(project.repo_folder.as_deref()).await.branch.is_some();
        PullRequestFlowSite { has_folder, has_branch }
    }

    async fn change_task_key(&self, tx: &Tx, project: &mut project::Model, requested: Option<&str>) -> Result<()> {
        let Some(requested) = requested else { return Ok(()) };
        let key = TaskKey::normalize(requested)?;
        if key == project.task_key {
            return Ok(());
        }
        require_key_free(tx, &key).await?;
        project.task_key = key;
        repo::task::recode_project(tx.db(), project.id, &project.task_key).await
    }

    /// Auto-commit only means something in a git repository: asked for anywhere else, it stays off.
    async fn auto_commit_allowed(&self, requested: Option<bool>, repo_folder: Option<&str>) -> bool {
        requested == Some(true) && self.repositories.is_repository(repo_folder).await
    }
}

/// A blank branch means the folder's current one. Anything git could read as an option or a revision
/// range is refused, since the name reaches git as an argument.
fn main_branch_of(raw: Option<&str>) -> Result<Option<String>> {
    let Some(branch) = raw.map(jstr::strip).filter(|branch| !branch.is_empty()) else { return Ok(None) };
    let forbidden = |character: char| character.is_whitespace() || character.is_control() || "~^:?*[\\".contains(character);
    if branch.starts_with('-') || branch.starts_with('/') || branch.ends_with('/') || branch.ends_with(".lock") || branch.contains("..") || branch.contains("@{") || branch.chars().any(forbidden) {
        return Err(RekallError::illegal(format!("'{branch}' is not a branch name git accepts. Use a name such as main or develop.")));
    }
    Ok(Some(branch.to_string()))
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

/// The key the request names, or one suggested from the label; either way nobody else holds it.
async fn chosen_task_key(tx: &Tx, requested: Option<&str>, label: &str) -> Result<String> {
    let Some(requested) = requested.filter(|key| !jstr::is_blank(key)) else {
        return Ok(TaskKey::suggest(label, &repo::project::list_keys(tx.db()).await?));
    };
    let key = TaskKey::normalize(requested)?;
    require_key_free(tx, &key).await?;
    Ok(key)
}

/// A key names one project across every company, or a task code would be ambiguous.
async fn require_key_free(tx: &Tx, key: &str) -> Result<()> {
    if repo::project::find_by_key(tx.db(), key).await?.is_some() {
        return Err(RekallError::conflict(format!("The key {key} belongs to another project. A task code has to name one task.")));
    }
    Ok(())
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

#[cfg(test)]
#[path = "../../../tests/unit/catalog/service/project_service_tests.rs"]
mod tests;
