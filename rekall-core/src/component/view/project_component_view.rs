use rekall_kernel::{Id, Instant};
use rekall_store::project_component;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectComponentView {
    pub id: Id,
    pub project_id: Id,
    pub name: String,
    pub folder: String,
    pub description: String,
    pub position: i32,
    pub created_at: Instant,
    pub updated_at: Instant,
}

impl ProjectComponentView {
    pub fn of(row: &project_component::Model) -> Self {
        Self {
            id: row.id,
            project_id: row.project_id,
            name: row.name.clone(),
            folder: row.folder.clone(),
            description: row.description.clone(),
            position: row.position,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
