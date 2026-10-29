use std::sync::Arc;

use rekall_core::claude::{ClaudeCodeSetup, Installation};
use rekall_kernel::Result;

/// Whether Claude Code knows Rekall, and the install that makes it so. Both run a process, so
/// each runs off the async workers.
#[derive(Clone)]
pub struct ClaudeInstallService {
    setup: Arc<dyn ClaudeCodeSetup>,
}

impl ClaudeInstallService {
    pub fn new(setup: Arc<dyn ClaudeCodeSetup>) -> Self {
        Self { setup }
    }

    pub async fn status(&self) -> Installation {
        let setup = self.setup.clone();
        tokio::task::spawn_blocking(move || setup.status()).await.expect("status never panics")
    }

    pub async fn install(&self) -> Result<Installation> {
        let setup = self.setup.clone();
        tokio::task::spawn_blocking(move || setup.install()).await.expect("install never panics")
    }
}

#[cfg(test)]
#[path = "../../tests/unit/claude/claude_install_service_tests.rs"]
mod tests;
