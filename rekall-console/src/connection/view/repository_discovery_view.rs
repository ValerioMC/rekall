use crate::connection::DiscoveredRepositoryView;

/// The repositories under a project's folder, in path order, each with what discovering it did.
#[derive(Clone, Debug)]
pub struct RepositoryDiscoveryView {
    pub folder: String,
    pub repositories: Vec<DiscoveredRepositoryView>,
}
