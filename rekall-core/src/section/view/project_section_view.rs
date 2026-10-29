use rekall_kernel::{Id, Instant};
use rekall_store::project_section;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSectionView {
    pub id: Id,
    pub project_id: Id,
    pub title: String,
    pub body_markdown: String,
    pub position: i32,
    pub created_at: Instant,
    pub updated_at: Instant,
}

impl ProjectSectionView {
    pub fn of(row: &project_section::Model) -> Self {
        Self {
            id: row.id,
            project_id: row.project_id,
            title: row.title.clone(),
            body_markdown: row.body_markdown.clone(),
            position: row.position,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
