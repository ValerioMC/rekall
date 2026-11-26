use std::sync::Arc;

use rekall_core::guard::{GuardInstaller, GuardProtection};
use rekall_kernel::Result;
use rekall_store::SessionProvider;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct GuardProtectionService {
    claude: Arc<Mutex<Arc<dyn GuardInstaller>>>,
    codex: Arc<Mutex<Arc<dyn GuardInstaller>>>,
}

impl GuardProtectionService {
    pub fn new(claude: Arc<dyn GuardInstaller>, codex: Arc<dyn GuardInstaller>) -> Self {
        Self {
            claude: Arc::new(Mutex::new(claude)),
            codex: Arc::new(Mutex::new(codex)),
        }
    }

    pub async fn status(&self, provider: SessionProvider) -> Result<GuardProtection> {
        self.installer(provider).lock().await.status().await
    }

    pub async fn install(&self, provider: SessionProvider) -> Result<GuardProtection> {
        self.installer(provider).lock().await.install().await
    }

    pub async fn uninstall(&self, provider: SessionProvider) -> Result<GuardProtection> {
        self.installer(provider).lock().await.uninstall().await
    }

    fn installer(&self, provider: SessionProvider) -> &Mutex<Arc<dyn GuardInstaller>> {
        match provider {
            SessionProvider::Claude => &self.claude,
            SessionProvider::Codex => &self.codex,
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/guard/service/guard_protection_service_tests.rs"]
mod tests;
