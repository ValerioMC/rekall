use serde::Serialize;

use super::DiscoveredRepositoryResponse;

/// The repositories under a project's folder, in path order, each with what discovering it did.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryDiscoveryResponse {
    pub folder: String,
    pub repositories: Vec<DiscoveredRepositoryResponse>,
}
