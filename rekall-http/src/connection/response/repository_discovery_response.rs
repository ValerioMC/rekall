use rekall_console::connection::RepositoryDiscoveryView;
use serde::Serialize;

use crate::connection::DiscoveredRepositoryResponse;

/// The repositories under a project's folder, in path order, each with what discovering it did.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryDiscoveryResponse {
    pub folder: String,
    pub repositories: Vec<DiscoveredRepositoryResponse>,
}

impl From<RepositoryDiscoveryView> for RepositoryDiscoveryResponse {
    fn from(view: RepositoryDiscoveryView) -> Self {
        Self {
            folder: view.folder,
            repositories: view.repositories.into_iter().map(DiscoveredRepositoryResponse::from).collect(),
        }
    }
}
