use serde::Deserialize;

use super::GraphStepRequest;

/// A deployment graph written whole: its title, what it is for, and every step in order, the
/// first where it begins.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DeploymentGraphRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub steps: Option<Vec<GraphStepRequest>>,
}
