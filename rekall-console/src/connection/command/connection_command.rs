use rekall_kernel::Id;

/// A connection to create or edit. On an edit, a field left out keeps what the connection has; on
/// a create, a label left out is made from the title. `kind` is fixed once created, and
/// `company_ids` is read on a create only: the companies of an existing connection change through
/// `ConnectionService::set_companies`.
#[derive(Clone, Debug, Default)]
pub struct ConnectionCommand {
    pub kind: Option<String>,
    pub label: Option<String>,
    pub title: Option<String>,
    pub base_url: Option<String>,
    /// Jira only; blank clears it.
    pub proxy_url: Option<String>,
    /// Grafana only: trust the self-signed certificate the instance serves.
    pub accept_self_signed: Option<bool>,
    /// GitLab and GitHub only: the group, namespace or owner the credential covers; blank clears it.
    pub root_path: Option<String>,
    pub company_ids: Option<Vec<Id>>,
}
