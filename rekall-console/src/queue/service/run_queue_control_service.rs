use std::sync::Arc;

use rekall_core::queue::{QueueRunner, RunQueueService, RunQueueView};
use rekall_kernel::{Id, Instant, Result};

use crate::queue::RunQueueSettingsCommand;

/// Edits the queue, and wakes its worker when a change is one it would otherwise wait a tick for.
#[derive(Clone)]
pub struct RunQueueControlService {
    queue: RunQueueService,
    runner: Arc<dyn QueueRunner>,
}

impl RunQueueControlService {
    pub fn new(queue: RunQueueService, runner: Arc<dyn QueueRunner>) -> Self {
        Self { queue, runner }
    }

    pub async fn view(&self) -> Result<RunQueueView> {
        self.queue.view().await
    }

    pub async fn update_settings(&self, command: RunQueueSettingsCommand) -> Result<RunQueueView> {
        self.queue.update_settings(command.ceiling_percent, command.skip_permissions, command.model.as_deref(), command.effort.as_deref()).await
    }

    pub async fn add(&self, task_id: Id) -> Result<RunQueueView> {
        let added = self.queue.add(task_id).await?;
        self.runner.nudge();
        Ok(added)
    }

    pub async fn remove(&self, item_id: Id) -> Result<RunQueueView> {
        self.queue.remove(item_id).await
    }

    pub async fn move_to(&self, item_id: Id, index: i32) -> Result<RunQueueView> {
        self.queue.move_to(item_id, index).await
    }

    pub async fn clear_settled(&self) -> Result<RunQueueView> {
        self.queue.clear_settled().await
    }

    pub async fn start(&self, start_at: Option<Instant>) -> Result<RunQueueView> {
        self.runner.start(&self.queue, start_at).await
    }

    pub async fn stop(&self) -> Result<RunQueueView> {
        self.runner.stop().await
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/queue/service/run_queue_control_service_tests.rs"]
mod tests;
