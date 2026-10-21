/// What asking the server with the credential found. Nothing here quotes the credential.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProbeOutcome {
    /// The credential was accepted; who it signs in as and what the server runs, when it said.
    Accepted { identity: Option<String>, version: Option<String> },
    /// The server answered 401 or 403.
    Rejected { status: u16 },
    /// No answer that says either way: the network, TLS, or an unexpected status.
    Unreachable { reason: String },
}
