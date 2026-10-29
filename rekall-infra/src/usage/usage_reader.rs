use async_trait::async_trait;

use super::ClaudeUsageView;

/// What the meter and the run queue read usage through.
#[async_trait]
pub trait UsageReader: Send + Sync {
    async fn current(&self) -> ClaudeUsageView;
    async fn refresh(&self) -> ClaudeUsageView;
    fn release_on_shutdown(&self) {}
}
