use async_trait::async_trait;
use rekall_kernel::{Instant, Result};

use super::{RunQueueService, RunQueueView};

/// The worker that works through the queue, one terminal at a time.
#[async_trait]
pub trait QueueRunner: Send + Sync {
    /// Wakes the worker to look at the queue now rather than at its next tick.
    fn nudge(&self);

    /// Starts the queue now, or arms it for `start_at`.
    async fn start(&self, queue: &RunQueueService, start_at: Option<Instant>) -> Result<RunQueueView>;

    /// Stops the queue, closing the running session and putting its item back at the head.
    async fn stop(&self) -> Result<RunQueueView>;
}
