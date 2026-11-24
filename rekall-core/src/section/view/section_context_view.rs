use rekall_kernel::{Id, Instant};
use rekall_store::section_context;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionContextView {
    pub id: Id,
    pub project_id: Id,
    pub title: String,
    pub position: i32,
    pub created_at: Instant,
    pub updated_at: Instant,
}

impl SectionContextView {
    pub fn of(row: &section_context::Model) -> Self {
        Self {
            id: row.id,
            project_id: row.project_id,
            title: row.title.clone(),
            position: row.position,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
