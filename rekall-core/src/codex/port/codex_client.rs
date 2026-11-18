use async_trait::async_trait;
use rekall_kernel::Result;

use crate::codex::{CodexModel, CodexSetup, CodexUsage};

#[async_trait]
pub trait CodexClient: Send + Sync {
    async fn models(&self) -> Result<Vec<CodexModel>>;
    async fn usage(&self, refresh: bool) -> CodexUsage;
    async fn setup(&self) -> Result<CodexSetup>;
    async fn install(&self) -> Result<CodexSetup>;
}
