use async_trait::async_trait;
use rekall_kernel::{Id, Result};

use super::{TerminalOpening, TerminalView};

/// The live `claude` terminals: opened on a task or project, listed, and closed.
#[async_trait]
pub trait TerminalHost: Send + Sync {
    async fn open(&self, opening: TerminalOpening) -> Result<TerminalView>;

    fn list(&self) -> Vec<TerminalView>;

    fn get(&self, id: Id) -> Option<TerminalView>;

    /// The terminal as it was when it closed, or none when no terminal has that id.
    async fn close(&self, id: Id, reason: &str) -> Option<TerminalView>;
}
