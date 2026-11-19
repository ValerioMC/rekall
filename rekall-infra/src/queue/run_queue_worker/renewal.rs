//! Opening a fresh session on the same item when the one in hand has spent its steps.

use rekall_kernel::Result;
use rekall_store::RunQueueItemState;
use tracing::{info, warn};

use super::Worker;

impl Worker {
    /// Close the terminal that ended its session and open another on the same item, which stays
    /// `Running`. Decided again here, after the grace: a session that went on to the next step
    /// by itself keeps its terminal.
    pub(super) async fn renew(&mut self, generation: u64) -> Result<()> {
        if !self.is_current(generation) {
            return Ok(());
        }
        let current = self.active_of(generation).expect("checked above");
        if current.closing {
            return Ok(());
        }
        let work = self.work.read(current.task_id).await?;
        if !self.is_spent(current.terminal_id, &work) {
            return Ok(());
        }
        self.mark_closing(generation);
        match self.terminals.renew(current.terminal_id).await {
            Ok(renewed) => {
                if let Some(active) = self.active_mut(generation) {
                    active.terminal_id = renewed.id;
                    active.closing = false;
                }
                info!(terminal_id = %renewed.id, "Run queue: a new session opened on the next step");
                Ok(())
            }
            Err(failure) => {
                warn!(%failure, task_id = %current.task_id, "Run queue: the next session could not open");
                self.forget(generation);
                self.release_running_steps(current.task_id).await;
                self.queue
                    .mark_item(current.item_id, RunQueueItemState::Failed, Some("The next session could not be opened."))
                    .await?;
                self.advance().await
            }
        }
    }
}
