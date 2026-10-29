/// A connection to create or edit. On an edit, a field left out keeps what the connection has; on
/// a create, a label left out is made from the title. `kind` is fixed once created.
#[derive(Clone, Debug, Default)]
pub struct ConnectionCommand {
    pub kind: Option<String>,
    pub label: Option<String>,
    pub title: Option<String>,
    pub base_url: Option<String>,
    /// Jira only; blank clears it.
    pub proxy_url: Option<String>,
}
