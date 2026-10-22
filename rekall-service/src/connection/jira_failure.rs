/// Why Jira gave no issues. Nothing here quotes the credential.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JiraFailure {
    /// Jira answered 401 or 403 to the stored credential.
    Rejected { status: u16 },
    /// The network, TLS, or an answer Jira explained with its own message.
    Failed { reason: String },
}
