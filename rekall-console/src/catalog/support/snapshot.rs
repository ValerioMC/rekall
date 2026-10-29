//! The rows a response is built from, loaded in a handful of queries rather than one per row.
//! Each loader reads what its responses need and nothing more: a company or project list never
//! reads a step, and a task list reads the steps of the tasks it lists.

use std::collections::{HashMap, HashSet};

use rekall_kernel::{Id, Instant, InternalFailure, RekallError, Result};
use rekall_store::prelude::*;
use rekall_store::{company, document_task, project, tag, task, task_step, wrapup};
use rekall_store::repository as repo;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

use crate::catalog::{CompanyView, ProjectView, TaskGraph, TaskRefView, TaskView};

#[derive(Default)]
pub struct Snapshot {
    pub companies: HashMap<Id, company::Model>,
    pub projects: HashMap<Id, project::Model>,
    pub tasks: HashMap<Id, task::Model>,
    steps: HashMap<Id, Vec<task_step::Model>>,
    tasks_with_wrapup: HashSet<Id>,
    tags: HashMap<Id, tag::Model>,
    documents_per_task: HashMap<Id, usize>,
    tasks_per_project: HashMap<Id, usize>,
    icon_image_versions: HashMap<Id, Instant>,
    logo_image_versions: HashMap<Id, Instant>,
}

impl Snapshot {
    /// Every company and project, with a task count per project and each icon and logo version:
    /// what the company and project lists answer from.
    pub async fn of_catalog(db: &impl ConnectionTrait) -> Result<Self> {
        Ok(Self {
            companies: Company::find().all(db).await?.into_iter().map(|c| (c.id, c)).collect(),
            projects: Project::find().all(db).await?.into_iter().map(|p| (p.id, p)).collect(),
            tasks_per_project: count_by(db, Task::find(), task::Column::ProjectId).await?,
            icon_image_versions: repo::project_icon_image::find_all_versions(db).await?.into_iter().collect(),
            logo_image_versions: repo::company_logo_image::find_all_versions(db).await?.into_iter().collect(),
            ..Self::default()
        })
    }

    /// The tasks, and what their rows need: projects, companies, steps, wrapups, tags, note counts.
    pub async fn for_tasks(db: &impl ConnectionTrait, tasks: &[task::Model]) -> Result<Self> {
        let mut snapshot = Self::with_parents(db, tasks.to_vec()).await?;
        let ids: Vec<Id> = tasks.iter().map(|task| task.id).collect();
        for step in TaskStep::find()
            .filter(task_step::Column::TaskId.is_in(ids.clone()))
            .order_by_asc(task_step::Column::Position)
            .all(db)
            .await?
        {
            snapshot.steps.entry(step.task_id).or_default().push(step);
        }
        let with_wrapup: Vec<Id> = Wrapup::find()
            .select_only()
            .column(wrapup::Column::TaskId)
            .filter(wrapup::Column::TaskId.is_in(ids.clone()))
            .into_tuple()
            .all(db)
            .await?;
        snapshot.tasks_with_wrapup = with_wrapup.into_iter().collect();
        let tag_ids: Vec<Id> = tasks.iter().filter_map(|task| task.tag_id).collect();
        snapshot.tags = Tag::find().filter(tag::Column::Id.is_in(tag_ids)).all(db).await?.into_iter().map(|t| (t.id, t)).collect();
        let links = DocumentTask::find().filter(document_task::Column::TaskId.is_in(ids));
        snapshot.documents_per_task = count_by(db, links, document_task::Column::TaskId).await?;
        Ok(snapshot)
    }

    pub async fn for_task(db: &impl ConnectionTrait, task: &task::Model) -> Result<Self> {
        Self::for_tasks(db, std::slice::from_ref(task)).await
    }

    /// The tasks with these ids, with their projects and companies: what a task reference needs.
    /// An id with no task is left out; a caller that requires it checks `tasks`.
    pub async fn for_task_refs(db: &impl ConnectionTrait, task_ids: &[Id]) -> Result<Self> {
        let tasks = repo::task::find_by_ids(db, task_ids.to_vec()).await?;
        Self::with_parents(db, tasks).await
    }

    /// One project and what its row needs.
    pub async fn for_project(db: &impl ConnectionTrait, project: &project::Model) -> Result<Self> {
        let mut snapshot = Self::default();
        let company = Company::find_by_id(project.company_id).one(db).await?.ok_or_else(missing)?;
        let tasks = Task::find().filter(task::Column::ProjectId.eq(project.id));
        snapshot.tasks_per_project = count_by(db, tasks, task::Column::ProjectId).await?;
        if let Some(version) = repo::project_icon_image::find_version(db, project.id).await? {
            snapshot.icon_image_versions.insert(project.id, version);
        }
        snapshot.companies.insert(company.id, company);
        snapshot.projects.insert(project.id, project.clone());
        Ok(snapshot)
    }

    async fn with_parents(db: &impl ConnectionTrait, tasks: Vec<task::Model>) -> Result<Self> {
        let mut snapshot = Self::default();
        let project_ids: Vec<Id> = tasks.iter().map(|task| task.project_id).collect();
        snapshot.projects = repo::project::find_by_ids(db, project_ids).await?.into_iter().map(|p| (p.id, p)).collect();
        let company_ids: Vec<Id> = snapshot.projects.values().map(|project| project.company_id).collect();
        snapshot.companies = Company::find()
            .filter(company::Column::Id.is_in(company_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|c| (c.id, c))
            .collect();
        snapshot.tasks = tasks.into_iter().map(|task| (task.id, task)).collect();
        Ok(snapshot)
    }

    pub fn task_response(&self, task: &task::Model) -> Result<TaskView> {
        let project = self.projects.get(&task.project_id).ok_or_else(missing)?;
        let company = self.companies.get(&project.company_id).ok_or_else(missing)?;
        let graph = TaskGraph {
            task,
            project,
            company,
            steps: self.steps.get(&task.id).map(|s| s.iter().collect()).unwrap_or_default(),
            document_count: self.documents_per_task.get(&task.id).copied().unwrap_or(0),
            has_wrapup: self.tasks_with_wrapup.contains(&task.id),
            tag: task.tag_id.and_then(|id| self.tags.get(&id)),
        };
        Ok(TaskView::of(&graph))
    }

    pub fn project_response(&self, project: &project::Model) -> Result<ProjectView> {
        let company = self.companies.get(&project.company_id).ok_or_else(missing)?;
        Ok(ProjectView::of(
            project,
            company,
            self.tasks_per_project.get(&project.id).copied().unwrap_or(0),
            self.icon_image_versions.get(&project.id).copied(),
        ))
    }

    pub fn company_response(&self, company: &company::Model) -> CompanyView {
        let projects: Vec<&project::Model> = self.projects.values().filter(|p| p.company_id == company.id).collect();
        let tasks = projects.iter().map(|p| self.tasks_per_project.get(&p.id).copied().unwrap_or(0)).sum();
        CompanyView::of(company, projects.len(), tasks, self.logo_image_versions.get(&company.id).copied())
    }

    pub fn task_ref(&self, task: &task::Model) -> Result<TaskRefView> {
        let project = self.projects.get(&task.project_id).ok_or_else(missing)?;
        let company = self.companies.get(&project.company_id).ok_or_else(missing)?;
        Ok(TaskRefView::of(task, project, company))
    }
}

/// How many rows `query` holds per value of `column`, counted in SQL.
async fn count_by<E, C>(db: &impl ConnectionTrait, query: sea_orm::Select<E>, column: C) -> Result<HashMap<Id, usize>>
where
    E: EntityTrait,
    C: ColumnTrait,
{
    let counts: Vec<(Id, i64)> = query
        .select_only()
        .column(column)
        .column_as(column.count(), "count")
        .group_by(column)
        .into_tuple()
        .all(db)
        .await?;
    Ok(counts.into_iter().map(|(id, count)| (id, count as usize)).collect())
}

fn missing() -> RekallError {
    RekallError::internal(InternalFailure::Integrity, "A row points at a parent that is not there")
}
