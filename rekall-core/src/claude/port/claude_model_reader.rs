use async_trait::async_trait;
use rekall_kernel::Result;

use crate::claude::ClaudeModel;

/// The models the installed Claude Code offers, read afresh on every call. A missing, unsupported
/// or silent CLI is a conflict.
#[async_trait]
pub trait ClaudeModelReader: Send + Sync {
    async fn models(&self) -> Result<Vec<ClaudeModel>>;
}
