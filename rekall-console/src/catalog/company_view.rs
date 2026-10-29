use rekall_common::{Id, Instant};
use rekall_model::company;

#[derive(Clone, Debug)]
pub struct CompanyView {
    pub id: Id,
    pub name: String,
    pub description: Option<String>,
    /// Set when the company has an uploaded logo; changes with every upload.
    pub logo_image_version: Option<Instant>,
    pub project_count: usize,
    pub task_count: usize,
    pub updated_at: Instant,
}

impl CompanyView {
    pub fn of(company: &company::Model, project_count: usize, task_count: usize, logo_image_version: Option<Instant>) -> Self {
        Self {
            id: company.id,
            name: company.name.clone(),
            description: company.description.clone(),
            logo_image_version,
            project_count,
            task_count,
            updated_at: company.updated_at,
        }
    }
}
