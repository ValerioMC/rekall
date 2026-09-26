use rekall_common::{Id, Instant};
use rekall_model::company;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyResponse {
    pub id: Id,
    pub name: String,
    pub description: Option<String>,
    pub project_count: usize,
    pub task_count: usize,
    pub updated_at: Instant,
}

impl CompanyResponse {
    pub fn of(company: &company::Model, project_count: usize, task_count: usize) -> Self {
        Self {
            id: company.id,
            name: company.name.clone(),
            description: company.description.clone(),
            project_count,
            task_count,
            updated_at: company.updated_at,
        }
    }
}
