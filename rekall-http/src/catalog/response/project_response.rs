use rekall_kernel::{Id, Instant};
use rekall_console::catalog::ProjectView;
use rekall_store::ProjectStatus;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectResponse {
    pub id: Id,
    pub label: String,
    pub title: String,
    pub status: ProjectStatus,
    pub icon: String,
    /// Set when the project has an uploaded picture; changes with every upload.
    pub icon_image_version: Option<Instant>,
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

impl From<ProjectView> for ProjectResponse {
    fn from(view: ProjectView) -> Self {
        Self {
            id: view.id,
            label: view.label,
            title: view.title,
            status: view.status,
            icon: view.icon,
            icon_image_version: view.icon_image_version,
            description: view.description,
            blueprint_markdown: view.blueprint_markdown,
            repo_folder: view.repo_folder,
            auto_commit: view.auto_commit,
            company_id: view.company_id,
            company_name: view.company_name,
            task_count: view.task_count,
            anchor: view.anchor,
            updated_at: view.updated_at,
        }
    }
}
