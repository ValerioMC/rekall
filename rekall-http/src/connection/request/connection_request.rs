use rekall_kernel::Id;
use rekall_console::connection::ConnectionCommand;
use serde::Deserialize;

/// A connection to create or edit. On an edit, a field left out keeps what the connection has; on
/// a create, a label left out is made from the title. `kind` is fixed once created, and
/// `companyIds` is read on a create only: an existing connection is shared through its own route.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ConnectionRequest {
    pub kind: Option<String>,
    pub label: Option<String>,
    pub title: Option<String>,
    pub base_url: Option<String>,
    /// Jira only; blank clears it.
    pub proxy_url: Option<String>,
    /// GitLab and GitHub only: the group, namespace or owner the credential covers; blank clears it.
    pub root_path: Option<String>,
    pub company_ids: Option<Vec<Id>>,
}

impl From<ConnectionRequest> for ConnectionCommand {
    fn from(request: ConnectionRequest) -> Self {
        Self {
            kind: request.kind,
            label: request.label,
            title: request.title,
            base_url: request.base_url,
            proxy_url: request.proxy_url,
            root_path: request.root_path,
            company_ids: request.company_ids,
        }
    }
}
