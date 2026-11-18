use std::sync::Arc;

use rekall_core::codex::{CodexClient, CodexModel, CodexSetup, CodexUsage};
use rekall_kernel::Result;

#[derive(Clone)]
pub struct CodexService {
    client: Arc<dyn CodexClient>,
}

impl CodexService {
    pub fn new(client: Arc<dyn CodexClient>) -> Self {
        Self { client }
    }

    pub async fn models(&self) -> Result<Vec<CodexModel>> {
        self.client.models().await
    }

    pub async fn usage(&self, refresh: bool) -> CodexUsage {
        self.client.usage(refresh).await
    }

    pub async fn setup(&self) -> Result<CodexSetup> {
        self.client.setup().await
    }

    pub async fn install(&self) -> Result<CodexSetup> {
        self.client.install().await
    }
}
