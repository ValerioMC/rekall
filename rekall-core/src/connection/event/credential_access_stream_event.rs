use rekall_kernel::Id;
use serde::Serialize;

/// Emitted with every line of a connection's access log, so the terminal pane shows a session
/// reaching a connection as it happens. It names the terminal and the connection, never the credential.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialAccessStreamEvent {
    pub terminal_id: Id,
    pub connection_id: Id,
    pub granted: bool,
}

impl crate::StreamEvent for CredentialAccessStreamEvent {
    const FRAME: &'static str = "credential-access";
}
