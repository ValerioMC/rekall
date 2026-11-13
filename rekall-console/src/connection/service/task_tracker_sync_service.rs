use rekall_core::connection::TaskTrackerService;
use rekall_kernel::{Id, Result};

use crate::catalog::{TaskService, TaskView};

/// Writes a task's title and description to its issue in the project's task manager, the one
/// write that is not automatic.
#[derive(Clone)]
pub struct TaskTrackerSyncService {
    tracker: TaskTrackerService,
    tasks: TaskService,
}

impl TaskTrackerSyncService {
    pub fn new(tracker: TaskTrackerService, tasks: TaskService) -> Self {
        Self { tracker, tasks }
    }

    pub async fn sync(&self, task_id: Id) -> Result<TaskView> {
        self.tracker.sync_content(task_id).await?;
        self.tasks.get(task_id).await
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/connection/service/task_tracker_sync_service_tests.rs"]
mod tests;
