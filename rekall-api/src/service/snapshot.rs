//! The rows a response is built from, loaded in a handful of queries rather than one per row.

use std::collections::HashMap;

use rekall_common::{Id, RekallError, Result};
use rekall_model::prelude::*;
use rekall_model::{company, document_task, project, tag, task, task_step, wrapup};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};

use crate::dto::{CompanyResponse, ProjectResponse, TaskGraph, TaskRef, TaskResponse};

#[derive(Default)]
pub struct Snapshot {
    pub companies: HashMap<Id, company::Model>,
    pub projects: HashMap<Id, project::Model>,
    pub tasks: HashMap<Id, task::Model>,
    pub steps: HashMap<Id, Vec<task_step::Model>>,
    pub wrapups: HashMap<Id, wrapup::Model>,
    pub tags: HashMap<Id, tag::Model>,
    pub documents_per_task: HashMap<Id, usize>,
    pub tasks_per_project: HashMap<Id, usize>,
}

impl Snapshot {
    /// Everything: what the list endpoints answer from.
    pub async fn load(db: &impl ConnectionTrait) -> Result<Self> {
        let mut snapshot = Self::default();
        for company in Company::find().all(db).await? {
            snapshot.companies.insert(company.id, company);
        }
        for project in Project::find().all(db).await? {
            snapshot.projects.insert(project.id, project);
        }
        for task in Task::find().all(db).await? {
            *snapshot.tasks_per_project.entry(task.project_id).or_default() += 1;
            snapshot.tasks.insert(task.id, task);
        }
        for step in TaskStep::find().order_by_asc(task_step::Column::Position).all(db).await? {
            snapshot.steps.entry(step.task_id).or_default().push(step);
        }
        for wrapup in Wrapup::find().all(db).await? {
            snapshot.wrapups.insert(wrapup.task_id, wrapup);
        }
        for tag in Tag::find().all(db).await? {
            snapshot.tags.insert(tag.id, tag);
        }
        for link in DocumentTask::find().all(db).await? {
            *snapshot.documents_per_task.entry(link.task_id).or_default() += 1;
        }
        Ok(snapshot)
    }

    /// One task and what its row needs.
    pub async fn for_task(db: &impl ConnectionTrait, task: &task::Model) -> Result<Self> {
        let mut snapshot = Self::default();
        let project = Project::find_by_id(task.project_id).one(db).await?.ok_or_else(missing)?;
        let company = Company::find_by_id(project.company_id).one(db).await?.ok_or_else(missing)?;
        let steps = TaskStep::find()
            .filter(task_step::Column::TaskId.eq(task.id))
            .order_by_asc(task_step::Column::Position)
            .all(db)
            .await?;
        snapshot.steps.insert(task.id, steps);
        if let Some(wrapup) = Wrapup::find().filter(wrapup::Column::TaskId.eq(task.id)).one(db).await? {
            snapshot.wrapups.insert(task.id, wrapup);
        }
        if let Some(tag_id) = task.tag_id {
            if let Some(tag) = Tag::find_by_id(tag_id).one(db).await? {
                snapshot.tags.insert(tag.id, tag);
            }
        }
        let links = DocumentTask::find().filter(document_task::Column::TaskId.eq(task.id)).all(db).await?;
        snapshot.documents_per_task.insert(task.id, links.len());
        snapshot.companies.insert(company.id, company);
        snapshot.projects.insert(project.id, project);
        snapshot.tasks.insert(task.id, task.clone());
        Ok(snapshot)
    }

    /// One project and what its row needs.
    pub async fn for_project(db: &impl ConnectionTrait, project: &project::Model) -> Result<Self> {
        let mut snapshot = Self::default();
        let company = Company::find_by_id(project.company_id).one(db).await?.ok_or_else(missing)?;
        let count = Task::find().filter(task::Column::ProjectId.eq(project.id)).all(db).await?.len();
        snapshot.tasks_per_project.insert(project.id, count);
        snapshot.companies.insert(company.id, company);
        snapshot.projects.insert(project.id, project.clone());
        Ok(snapshot)
    }

    pub fn task_response(&self, task: &task::Model) -> Result<TaskResponse> {
        let project = self.projects.get(&task.project_id).ok_or_else(missing)?;
        let company = self.companies.get(&project.company_id).ok_or_else(missing)?;
        let graph = TaskGraph {
            task,
            project,
            company,
            steps: self.steps.get(&task.id).map(|s| s.iter().collect()).unwrap_or_default(),
            document_count: self.documents_per_task.get(&task.id).copied().unwrap_or(0),
            wrapup: self.wrapups.get(&task.id),
            tag: task.tag_id.and_then(|id| self.tags.get(&id)),
        };
        Ok(TaskResponse::of(&graph))
    }

    pub fn project_response(&self, project: &project::Model) -> Result<ProjectResponse> {
        let company = self.companies.get(&project.company_id).ok_or_else(missing)?;
        Ok(ProjectResponse::of(project, company, self.tasks_per_project.get(&project.id).copied().unwrap_or(0)))
    }

    pub fn company_response(&self, company: &company::Model) -> CompanyResponse {
        let projects: Vec<&project::Model> = self.projects.values().filter(|p| p.company_id == company.id).collect();
        let tasks = projects.iter().map(|p| self.tasks_per_project.get(&p.id).copied().unwrap_or(0)).sum();
        CompanyResponse::of(company, projects.len(), tasks)
    }

    pub fn task_ref(&self, task: &task::Model) -> Result<TaskRef> {
        let project = self.projects.get(&task.project_id).ok_or_else(missing)?;
        let company = self.companies.get(&project.company_id).ok_or_else(missing)?;
        Ok(TaskRef::of(task, project, company))
    }
}

fn missing() -> RekallError {
    RekallError::internal("IllegalStateException", "A row points at a parent that is not there")
}
