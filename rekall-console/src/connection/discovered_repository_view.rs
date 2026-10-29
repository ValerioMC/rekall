use rekall_kernel::Id;

use super::DiscoveryOutcome;

/// One repository found under a project's folder, and what discovering it did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveredRepositoryView {
    pub folder: String,
    /// `host/group/name`, never the remote's URL, which may carry a credential.
    pub origin: Option<String>,
    pub outcome: DiscoveryOutcome,
    /// The company connection at the origin's host.
    pub connection_label: Option<String>,
    /// The binding that reaches it, made now or found.
    pub binding_id: Option<Id>,
    pub label: Option<String>,
    pub target: Option<String>,
    /// The component made for its folder, when one was.
    pub component: Option<String>,
}

impl DiscoveredRepositoryView {
    pub(super) fn unmatched(folder: String, origin: Option<String>, outcome: DiscoveryOutcome, connection_label: Option<String>) -> Self {
        Self { folder, origin, outcome, connection_label, binding_id: None, label: None, target: None, component: None }
    }
}
