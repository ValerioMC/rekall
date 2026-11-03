use std::sync::Arc;

use rekall_core::claude::{GuardInstaller, GuardProtection};
use rekall_kernel::Result;

/// Whether Claude Code runs the guard hooks, and the writes that add or remove them. Each reads or
/// writes a file, so each runs off the async workers.
#[derive(Clone)]
pub struct GuardProtectionService {
    installer: Arc<dyn GuardInstaller>,
}

impl GuardProtectionService {
    pub fn new(installer: Arc<dyn GuardInstaller>) -> Self {
        Self { installer }
    }

    pub async fn status(&self) -> GuardProtection {
        let installer = self.installer.clone();
        tokio::task::spawn_blocking(move || installer.status()).await.expect("status never panics")
    }

    pub async fn install(&self) -> Result<GuardProtection> {
        let installer = self.installer.clone();
        tokio::task::spawn_blocking(move || installer.install()).await.expect("install never panics")
    }

    pub async fn uninstall(&self) -> Result<GuardProtection> {
        let installer = self.installer.clone();
        tokio::task::spawn_blocking(move || installer.uninstall()).await.expect("uninstall never panics")
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/claude/service/guard_protection_service_tests.rs"]
mod tests;
