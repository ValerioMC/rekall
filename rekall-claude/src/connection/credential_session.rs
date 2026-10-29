use std::path::PathBuf;

use rekall_kernel::Id;
use rekall_core::claude::TerminalConnection;
use rekall_core::connection::Secret;

/// One terminal's access to its project's connections: the token its helpers present, the folder
/// they live in, and the connections they may ask for. It lives exactly as long as the terminal.
pub(super) struct CredentialSession {
    pub task_id: Option<Id>,
    pub token: Secret,
    pub folder: PathBuf,
    pub connections: Vec<TerminalConnection>,
}

impl CredentialSession {
    pub fn connection(&self, label: &str) -> Option<&TerminalConnection> {
        self.connections.iter().find(|connection| connection.label == label)
    }

    /// Compared in constant time, so the answer's timing says nothing about how much of a guess was right.
    pub fn accepts(&self, presented: Option<&str>) -> bool {
        let Some(presented) = presented else { return false };
        let expected = self.token.expose().as_bytes();
        let presented = presented.as_bytes();
        if expected.len() != presented.len() {
            return false;
        }
        expected.iter().zip(presented).fold(0u8, |difference, (a, b)| difference | (a ^ b)) == 0
    }
}
