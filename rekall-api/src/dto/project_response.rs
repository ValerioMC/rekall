use rekall_common::{Id, Instant};
use rekall_model::{company, project};
use rekall_model::ProjectStatus;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectResponse {
    pub id: Id,
    pub label: String,
    pub title: String,
    pub status: ProjectStatus,
    pub icon: String,
    pub description: Option<String>,
    pub blueprint_markdown: Option<String>,
    pub repo_folder: Option<String>,
    pub auto_commit: bool,
    pub company_id: Id,
    pub company_name: String,
    pub task_count: usize,
    pub anchor: String,
    pub updated_at: Instant,
}

impl ProjectResponse {
    pub fn of(project: &project::Model, company: &company::Model, task_count: usize) -> Self {
        Self {
            id: project.id,
            label: project.label.clone(),
            title: project.title.clone(),
            status: project.status,
            icon: project.icon.clone(),
            description: project.description.clone(),
            blueprint_markdown: project.blueprint_markdown.clone(),
            repo_folder: project.repo_folder.clone(),
            auto_commit: project.auto_commit,
            company_id: company.id,
            company_name: company.name.clone(),
            task_count,
            anchor: format!("project:{}", project.label),
            updated_at: project.updated_at,
        }
    }
}
