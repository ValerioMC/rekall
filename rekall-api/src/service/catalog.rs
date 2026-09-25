//! Companies, projects and tasks: created, edited, moved and deleted from the console.

use rekall_common::{jstr, Id, RekallError, Result};
use rekall_model::constraints::Phase;
use rekall_model::{company, document, project, task, ProjectStatus, RevisionKind, Slug, TaskStatus, TaskStepState};
use rekall_repository::repository as repo;
use rekall_service::revision::RevisionTrigger;
use rekall_service::{in_read, in_write, Services, Tx};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};

use super::Snapshot;
use crate::dto::{
    CompanyRequest, CompanyResponse, ProjectRepositoryResponse, ProjectRequest, ProjectResponse, TaskRequest, TaskResponse,
};

#[derive(Clone)]
pub struct CatalogService {
    services: Services,
}

impl CatalogService {
    pub fn new(services: Services) -> Self {
        Self { services }
    }

    fn ctx(&self) -> &rekall_service::Ctx {
        &self.services.ctx
    }

    // ------------------------------------------------------------------------------ companies

    pub async fn list_companies(&self) -> Result<Vec<CompanyResponse>> {
        in_read!(self.ctx(), |tx| {
            let snapshot = Snapshot::load(tx.db()).await?;
            let companies = repo::company::find_all_by_order_by_name_asc(tx.db()).await?;
            Ok::<_, RekallError>(companies.iter().map(|c| snapshot.company_response(c)).collect())
        })
    }

    pub async fn create_company(&self, request: CompanyRequest) -> Result<CompanyResponse> {
        in_write!(self.ctx(), |tx| {
            let now = self.ctx().now();
            let company = company::Model {
                id: Id::random(),
                name: request.name.clone().unwrap_or_default(),
                description: request.description.clone(),
                created_at: now,
                updated_at: now,
            };
            company.validate(Phase::Persist)?;
            company.clone().into_active_model().insert(tx.db()).await?;
            Ok::<_, RekallError>(CompanyResponse::of(&company, 0, 0))
        })
    }

    pub async fn update_company(&self, id: Id, request: CompanyRequest) -> Result<CompanyResponse> {
        in_write!(self.ctx(), |tx| {
            let before = require_company(&tx, Some(id)).await?;
            let mut company = before.clone();
            company.name = request.name.clone().unwrap_or_default();
            company.description = request.description.clone();
            if company != before {
                company.updated_at = self.ctx().now();
                company.validate(Phase::Update)?;
                company.clone().into_active_model().reset_all().update(tx.db()).await?;
            }
            let snapshot = Snapshot::load(tx.db()).await?;
            Ok::<_, RekallError>(snapshot.company_response(&company))
        })
    }

    pub async fn delete_company(&self, id: Id) -> Result<()> {
        in_write!(self.ctx(), |tx| {
            let company = require_company(&tx, Some(id)).await?;
            company::Entity::delete_by_id(company.id).exec(tx.db()).await?;
            sweep_orphans(&tx).await
        })
    }

    // ------------------------------------------------------------------------------ projects

    pub async fn list_projects(&self) -> Result<Vec<ProjectResponse>> {
        in_read!(self.ctx(), |tx| {
            let snapshot = Snapshot::load(tx.db()).await?;
            let projects = repo::project::find_all_by_order_by_company_name_asc_label_asc(tx.db()).await?;
            projects.iter().map(|p| snapshot.project_response(p)).collect::<Result<Vec<_>>>()
        })
    }

    pub async fn get_project(&self, id: Id) -> Result<ProjectResponse> {
        in_read!(self.ctx(), |tx| {
            let project = require_project(&tx, Some(id)).await?;
            Snapshot::for_project(tx.db(), &project).await?.project_response(&project)
        })
    }

    pub async fn create_project(&self, request: ProjectRequest) -> Result<ProjectResponse> {
        in_write!(self.ctx(), |tx| {
            let label = Slug::of(request.label.as_deref())?;
            let now = self.ctx().now();
            let mut project = project::Model {
                id: Id::random(),
                label,
                title: jstr::trim(request.title.as_deref().unwrap_or_default()).to_string(),
                status: ProjectStatus::Active,
                icon: "folder".to_string(),
                description: None,
                blueprint_markdown: None,
                repo_folder: None,
                auto_commit: false,
                company_id: Id::random(),
                created_at: now,
                updated_at: now,
            };
            self.apply_to_project(&tx, &mut project, &request).await?;
            project.validate(Phase::Persist)?;
            project.clone().into_active_model().insert(tx.db()).await?;
            Snapshot::for_project(tx.db(), &project).await?.project_response(&project)
        })
    }

    pub async fn update_project(&self, id: Id, request: ProjectRequest) -> Result<ProjectResponse> {
        in_write!(self.ctx(), |tx| {
            let before = require_project(&tx, Some(id)).await?;
            let mut project = before.clone();
            project.label = Slug::of(request.label.as_deref())?;
            project.title = jstr::trim(request.title.as_deref().unwrap_or_default()).to_string();
            self.apply_to_project(&tx, &mut project, &request).await?;
            if project != before {
                project.updated_at = self.ctx().now();
                project.validate(Phase::Update)?;
                project.clone().into_active_model().reset_all().update(tx.db()).await?;
            }
            Snapshot::for_project(tx.db(), &project).await?.project_response(&project)
        })
    }

    pub async fn delete_project(&self, id: Id) -> Result<()> {
        in_write!(self.ctx(), |tx| {
            let project = require_project(&tx, Some(id)).await?;
            project::Entity::delete_by_id(project.id).exec(tx.db()).await?;
            sweep_orphans(&tx).await
        })
    }

    /// The folder as git sees it, plus whether the project auto-commits there.
    pub async fn get_project_repository(&self, id: Id) -> Result<ProjectRepositoryResponse> {
        in_read!(self.ctx(), |tx| {
            let project = require_project(&tx, Some(id)).await?;
            let status = self.services.repositories.inspect(project.repo_folder.as_deref()).await;
            Ok::<_, RekallError>(ProjectRepositoryResponse::of(status, project.auto_commit))
        })
    }

    async fn apply_to_project(&self, tx: &Tx, project: &mut project::Model, request: &ProjectRequest) -> Result<()> {
        project.description = request.description.clone();
        project.blueprint_markdown = request.blueprint_markdown.clone();
        project.repo_folder = match request.repo_folder.as_deref() {
            None => None,
            Some(folder) if jstr::is_blank(folder) => None,
            Some(folder) => Some(jstr::trim(folder).to_string()),
        };
        // Auto-commit only means something in a git repository: asked for anywhere else, it stays off.
        project.auto_commit = request.auto_commit == Some(true)
            && self.services.repositories.is_repository(project.repo_folder.as_deref()).await;
        project.status = request.status.unwrap_or(ProjectStatus::Active);
        project.icon = match request.icon.as_deref() {
            Some(icon) if !jstr::is_blank(icon) => jstr::trim(icon).to_string(),
            _ => "folder".to_string(),
        };
        project.company_id = require_company_for_project(tx, request.company_id).await?.id;
        Ok(())
    }

    // ------------------------------------------------------------------------------ tasks

    pub async fn list_tasks(&self, project_id: Option<Id>) -> Result<Vec<TaskResponse>> {
        in_read!(self.ctx(), |tx| {
            let found = match project_id {
                None => repo::task::find_all_by_order_by_project_label_asc_label_asc(tx.db()).await?,
                Some(project_id) => repo::task::find_by_project_id_order_by_label_asc(tx.db(), project_id).await?,
            };
            let snapshot = Snapshot::load(tx.db()).await?;
            found.iter().map(|t| snapshot.task_response(t)).collect::<Result<Vec<_>>>()
        })
    }

    pub async fn get_task(&self, id: Id) -> Result<TaskResponse> {
        in_read!(self.ctx(), |tx| {
            let task = require_task(&tx, id).await?;
            task_response(&tx, &task).await
        })
    }

    pub async fn create_task(&self, request: TaskRequest) -> Result<TaskResponse> {
        in_write!(self.ctx(), |tx| {
            let label = Slug::of(request.label.as_deref())?;
            let title = jstr::trim(request.title.as_deref().unwrap_or_default()).to_string();
            let mut task = task::Model::new(label, title, Id::random(), self.ctx().now());
            apply_to_task(&tx, &mut task, &request).await?;
            task.validate(Phase::Persist)?;
            task.clone().into_active_model().insert(tx.db()).await?;
            task_response(&tx, &task).await
        })
    }

    pub async fn update_task(&self, id: Id, request: TaskRequest) -> Result<TaskResponse> {
        in_write!(self.ctx(), |tx| {
            let before = require_task(&tx, id).await?;
            let previous_status = before.status;
            self.keep_description_if_replaced(&mut tx, &before, request.description.as_deref(), RevisionTrigger::HandEdit)
                .await?;
            let mut task = before.clone();
            task.label = Slug::of(request.label.as_deref())?;
            task.title = jstr::trim(request.title.as_deref().unwrap_or_default()).to_string();
            apply_to_task(&tx, &mut task, &request).await?;
            if task != before {
                task.updated_at = self.ctx().now();
                task.validate(Phase::Update)?;
                task.clone().into_active_model().reset_all().update(tx.db()).await?;
            }
            let response = task_response(&tx, &task).await?;
            if previous_status != TaskStatus::Done && task.status == TaskStatus::Done {
                self.services.time_entries.stop_if_running_in(&mut tx, id).await?;
            }
            Ok(response)
        })
    }

    /// Writes an earlier revision back as the task's description, keeping the one it replaces.
    pub async fn restore_description_in(&self, tx: &mut Tx, id: Id, description: &str) -> Result<TaskResponse> {
        let before = require_task(tx, id).await?;
        self.keep_description_if_replaced(tx, &before, Some(description), RevisionTrigger::Restore).await?;
        let mut task = before.clone();
        task.description = Some(description.to_string());
        if task != before {
            task.updated_at = self.ctx().now();
            task.validate(Phase::Update)?;
            task.clone().into_active_model().reset_all().update(tx.db()).await?;
        }
        task_response(tx, &task).await
    }

    async fn keep_description_if_replaced(
        &self,
        tx: &mut Tx,
        task: &task::Model,
        incoming: Option<&str>,
        trigger: RevisionTrigger,
    ) -> Result<()> {
        if task.description.as_deref() != incoming {
            self.services
                .revisions
                .keep_in(tx, task.id, RevisionKind::Description, task.description.as_deref(), None, None, trigger)
                .await?;
        }
        Ok(())
    }

    /// `DONE` accepts a stepless task, `OPEN` sends it back; other states are refused.
    pub async fn review_task(&self, id: Id, target: TaskStepState, note: Option<&str>) -> Result<TaskResponse> {
        in_write!(self.ctx(), |tx| {
            require_task(&tx, id).await?;
            match target {
                TaskStepState::Done => {
                    self.services.review.accept_in(&mut tx, id).await?;
                }
                TaskStepState::Open => {
                    self.services.review.send_back_in(&mut tx, id, note).await?;
                }
                _ => {
                    return Err(RekallError::illegal(
                        "The console can accept a task (DONE) or send it back (OPEN). RUNNING follows a live session \
                         and CLAIMED is set when a Claude-authored wrapup lands.",
                    ))
                }
            }
            let task = require_task(&tx, id).await?;
            task_response(&tx, &task).await
        })
    }

    pub async fn delete_task(&self, id: Id) -> Result<()> {
        in_write!(self.ctx(), |tx| {
            let task = require_task(&tx, id).await?;
            task::Entity::delete_by_id(task.id).exec(tx.db()).await?;
            sweep_orphans(&tx).await
        })
    }

    pub async fn require_task_in(&self, tx: &Tx, id: Id) -> Result<task::Model> {
        require_task(tx, id).await
    }
}

async fn apply_to_task(tx: &Tx, task: &mut task::Model, request: &TaskRequest) -> Result<()> {
    task.description = request.description.clone();
    task.status = request.status.unwrap_or(TaskStatus::Todo);
    task.project_id = require_project(tx, request.project_id).await?.id;
    task.tag_id = match request.tag_id {
        None => None,
        Some(tag_id) => Some(
            repo::tag::find_by_id(tx.db(), tag_id)
                .await?
                .ok_or_else(|| RekallError::not_found("Tag", tag_id))?
                .id,
        ),
    };
    Ok(())
}

async fn task_response(tx: &Tx, task: &task::Model) -> Result<TaskResponse> {
    Snapshot::for_task(tx.db(), task).await?.task_response(task)
}

/// Notes attached to nothing are unreachable: every delete that can leave one sweeps them up.
async fn sweep_orphans(tx: &Tx) -> Result<()> {
    let orphans: Vec<Id> = repo::document::find_orphans(tx.db()).await?.into_iter().map(|d| d.id).collect();
    if !orphans.is_empty() {
        document::Entity::delete_many().filter(document::Column::Id.is_in(orphans)).exec(tx.db()).await?;
    }
    Ok(())
}

async fn require_company(tx: &Tx, id: Option<Id>) -> Result<company::Model> {
    let Some(id) = id else {
        return Err(RekallError::not_found_msg("A project must belong to a company"));
    };
    repo::company::find_by_id(tx.db(), id).await?.ok_or_else(|| RekallError::not_found("Company", id))
}

async fn require_company_for_project(tx: &Tx, id: Option<Id>) -> Result<company::Model> {
    require_company(tx, id).await
}

async fn require_project(tx: &Tx, id: Option<Id>) -> Result<project::Model> {
    let Some(id) = id else {
        return Err(RekallError::not_found_msg("A task must belong to a project"));
    };
    repo::project::find_by_id(tx.db(), id).await?.ok_or_else(|| RekallError::not_found("Project", id))
}

pub(crate) async fn require_task(tx: &Tx, id: Id) -> Result<task::Model> {
    repo::task::find_by_id(tx.db(), id).await?.ok_or_else(|| RekallError::not_found("Task", id))
}
