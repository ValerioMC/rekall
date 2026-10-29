use rekall_kernel::{Id, Instant};
use rekall_console::graph::DeploymentGraphView;
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
    /// Laid on every task of the project the console runs.
    pub release_default: bool,
    pub created_at: Instant,
    pub updated_at: Instant,
}

impl From<DeploymentGraphView> for DeploymentGraphResponse {
    fn from(view: DeploymentGraphView) -> Self {
        Self {
            id: view.id,
            project_id: view.project_id,
            title: view.title,
            description: view.description,
            steps: view.steps.into_iter().map(GraphStepResponse::from).collect(),
            problem: view.problem,
            release_default: view.release_default,
            created_at: view.created_at,
            updated_at: view.updated_at,
        }
    }
}
