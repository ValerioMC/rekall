use rekall_kernel::{Id, Instant};
use rekall_console::catalog::CompanyView;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyResponse {
    pub id: Id,
    pub name: String,
    pub description: Option<String>,
    /// Set when the company has an uploaded logo; changes with every upload.
    pub logo_image_version: Option<Instant>,
    pub project_count: usize,
    pub task_count: usize,
    pub updated_at: Instant,
}

impl From<CompanyView> for CompanyResponse {
    fn from(view: CompanyView) -> Self {
        Self {
            id: view.id,
            name: view.name,
            description: view.description,
            logo_image_version: view.logo_image_version,
            project_count: view.project_count,
            task_count: view.task_count,
            updated_at: view.updated_at,
        }
    }
}
