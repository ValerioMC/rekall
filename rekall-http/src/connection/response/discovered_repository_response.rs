use rekall_kernel::Id;
use rekall_console::connection::DiscoveredRepositoryView;
use serde::Serialize;

use crate::connection::DiscoveryOutcomeResponse;

/// One repository found under a project's folder, and what discovering it did.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredRepositoryResponse {
    pub folder: String,
    /// `host/group/name`, never the remote's URL, which may carry a credential.
    pub origin: Option<String>,
    pub outcome: DiscoveryOutcomeResponse,
    /// The company connection at the origin's host.
    pub connection_label: Option<String>,
    /// The binding that reaches it, made now or found.
    pub binding_id: Option<Id>,
    pub label: Option<String>,
    pub target: Option<String>,
    /// The component made for its folder, when one was.
    pub component: Option<String>,
}

impl From<DiscoveredRepositoryView> for DiscoveredRepositoryResponse {
    fn from(view: DiscoveredRepositoryView) -> Self {
        Self {
            folder: view.folder,
            origin: view.origin,
            outcome: view.outcome.into(),
            connection_label: view.connection_label,
            binding_id: view.binding_id,
            label: view.label,
            target: view.target,
            component: view.component,
        }
    }
}
