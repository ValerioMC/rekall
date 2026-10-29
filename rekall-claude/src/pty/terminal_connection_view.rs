use rekall_kernel::Id;
use rekall_store::ConnectionKind;
use rekall_core::claude::TerminalConnection;
use serde::Serialize;

/// A connection a terminal's project binds, as the terminal pane names it: never its credential.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalConnectionView {
    pub connection_id: Id,
    pub label: String,
    pub kind: ConnectionKind,
    /// Without one, the terminal's helpers cannot reach it.
    pub credential_set: bool,
}

impl TerminalConnectionView {
    pub fn of(connection: &TerminalConnection) -> Self {
        Self { connection_id: connection.connection_id, label: connection.label.clone(), kind: connection.kind, credential_set: connection.credential_set }
    }
}
