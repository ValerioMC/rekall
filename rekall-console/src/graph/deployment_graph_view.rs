use rekall_common::{Id, Instant};
use rekall_model::deployment_graph;

use super::GraphStepView;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeploymentGraphView {
    pub id: Id,
    pub project_id: Id,
    pub title: String,
    pub description: String,
    pub steps: Vec<GraphStepView>,
    /// Why the graph cannot run unattended yet, so it cannot be attached; `None` when it can.
    pub problem: Option<String>,
    /// Laid on every task of the project the console runs.
    pub release_default: bool,
    pub created_at: Instant,
    pub updated_at: Instant,
}

impl DeploymentGraphView {
    pub fn of(graph: &deployment_graph::Model, steps: Vec<GraphStepView>, problem: Option<String>) -> Self {
        Self {
            id: graph.id,
            project_id: graph.project_id,
            title: graph.title.clone(),
            description: graph.description.clone(),
            steps,
            problem,
            release_default: graph.release_default,
            created_at: graph.created_at,
            updated_at: graph.updated_at,
        }
    }
}
