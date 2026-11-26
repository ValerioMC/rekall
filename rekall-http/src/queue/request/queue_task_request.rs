use rekall_console::queue::QueueTaskCommand;
use rekall_core::queue::QueueLaunch;
use rekall_kernel::{Id, RekallError, Result};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::queue) struct QueueTaskRequest {
    task_id: Option<Id>,
    #[serde(flatten)]
    launch: QueueLaunch,
}

impl QueueTaskRequest {
    pub(in crate::queue) fn command(self) -> Result<QueueTaskCommand> {
        Ok(QueueTaskCommand {
            task_id: self.task_id.ok_or_else(|| RekallError::illegal("Name the task to queue."))?,
            launch: self.launch,
        })
    }
}
