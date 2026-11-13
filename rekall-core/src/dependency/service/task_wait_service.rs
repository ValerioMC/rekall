use rekall_kernel::{Id, Result};
use rekall_store::repository as repo;

use crate::claude::TaskWorkService;
use crate::dependency::TaskWait;
use crate::{load, Tx};

pub struct TaskWaitService;

impl TaskWaitService {
    /// Every task `task_id` depends on, met or not, in the order the dependencies were added.
    pub async fn of_task_in(tx: &Tx, task_id: Id) -> Result<Vec<TaskWait>> {
        let mut waits = Vec::new();
        for dependency in repo::task_dependency::list_of_task(tx.db(), task_id).await? {
            waits.push(Self::wait_for_in(tx, dependency.depends_on_task_id).await?);
        }
        Ok(waits)
    }

    async fn wait_for_in(tx: &Tx, task_id: Id) -> Result<TaskWait> {
        let task = load::task_or_unknown(tx.db(), task_id).await?;
        let work = TaskWorkService::read_in(tx, task_id).await?;
        Ok(TaskWait {
            task_id,
            anchor: task.anchor(),
            title: task.title,
            status: task.status,
            remaining: work.remaining,
        })
    }
}
