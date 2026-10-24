use rekall_common::{Id, Instant};
use rekall_model::deployment_graph;
use serde::Serialize;

use super::GraphStepResponse;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentGraphResponse {
    pub id: Id,
    pub project_id: Id,
    pub title: String,
    pub description: String,
    pub steps: Vec<GraphStepResponse>,
    /// Why the graph cannot run unattended yet, so it cannot be attached; `None` when it can.
    pub problem: Option<String>,
    pub created_at: Instant,
    pub updated_at: Instant,
}

impl DeploymentGraphResponse {
    pub fn of(graph: &deployment_graph::Model, steps: Vec<GraphStepResponse>, problem: Option<String>) -> Self {
        Self {
            id: graph.id,
            project_id: graph.project_id,
            title: graph.title.clone(),
            description: graph.description.clone(),
            steps,
            problem,
            created_at: graph.created_at,
            updated_at: graph.updated_at,
        }
    }
}
