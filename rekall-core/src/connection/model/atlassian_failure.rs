/// Why a Jira or Confluence site gave no answer. Nothing here quotes the credential.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AtlassianFailure {
    /// The site answered 401 or 403 to the stored credential.
    Rejected { status: u16 },
    /// The network, TLS, or an answer the site explained with its own message.
    Failed { reason: String },
}
