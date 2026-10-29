use rekall_console::graph::{DeploymentGraphCommand, GraphStepCommand};
use serde::Deserialize;

use crate::graph::GraphStepRequest;

/// A deployment graph written whole: its title, what it is for, and every step in order, the
/// first where it begins.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DeploymentGraphRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub steps: Option<Vec<GraphStepRequest>>,
}

impl From<DeploymentGraphRequest> for DeploymentGraphCommand {
    fn from(request: DeploymentGraphRequest) -> Self {
        Self {
            title: request.title,
            description: request.description,
            steps: request.steps.map(|steps| steps.into_iter().map(GraphStepCommand::from).collect()),
        }
    }
}
