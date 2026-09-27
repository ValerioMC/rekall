use rekall_common::{Id, Instant};
use rekall_model::diagram;
use serde::Serialize;

/// A diagram as a list shows it: everything but the graph.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagramSummaryView {
    pub id: Id,
    pub project_id: Id,
    pub task_id: Option<Id>,
    pub title: String,
    pub question: String,
    pub node_count: i32,
    pub edge_count: i32,
    pub created_at: Instant,
    pub updated_at: Instant,
}

impl DiagramSummaryView {
    pub fn of(row: &diagram::Model) -> Self {
        Self {
            id: row.id,
            project_id: row.project_id,
            task_id: row.task_id,
            title: row.title.clone(),
            question: row.question.clone(),
            node_count: row.node_count,
            edge_count: row.edge_count,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
