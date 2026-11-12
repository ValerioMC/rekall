//! The console's edits of the list: adding a task, taking one off, reordering, clearing what settled.

use rekall_kernel::{Id, RekallError, Result};
use rekall_store::repository as repo;
use rekall_store::{run_queue_item, RunQueueItemState, RunQueueState};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};

use crate::queue::RunQueueView;
use crate::{in_write, Tx};

use super::{ordered, renumber, require_item, task_title, RunQueueService};

impl RunQueueService {
    pub async fn add(&self, task_id: Id) -> Result<RunQueueView> {
        in_write!(&self.ctx, |tx| {
            self.enqueue(&tx, task_id).await?;
            let queue = self.queue(&tx).await?;
            self.publish(&mut tx, queue).await
        })
    }

    /// Add the task and, when the queue is idle, start it in the same transaction. A running,
    /// holding or scheduled queue only gains the item: it runs when the worker reaches it.
    pub async fn add_and_arm(&self, task_id: Id) -> Result<RunQueueView> {
        in_write!(&self.ctx, |tx| {
            self.enqueue(&tx, task_id).await?;
            let mut queue = self.queue(&tx).await?;
            if queue.state == RunQueueState::Idle {
                queue.move_to(RunQueueState::Running);
            }
            self.publish(&mut tx, queue).await
        })
    }

    pub async fn remove(&self, item_id: Id) -> Result<RunQueueView> {
        in_write!(&self.ctx, |tx| {
            let item = require_item(&tx, item_id).await?;
            if item.state == RunQueueItemState::Running {
                let title = task_title(&tx, item.task_id).await?;
                return Err(RekallError::conflict(format!("'{title}' is running. Stop the queue to take it off.")));
            }
            run_queue_item::Entity::delete_by_id(item.id).exec(tx.db()).await?;
            renumber(&tx, ordered(&tx).await?).await?;
            let queue = self.queue(&tx).await?;
            self.publish(&mut tx, queue).await
        })
    }

    /// Move a waiting item to `index` among the waiting items. Settled and running items keep
    /// their places ahead of them; only the order still to run changes.
    pub async fn move_to(&self, item_id: Id, index: i32) -> Result<RunQueueView> {
        in_write!(&self.ctx, |tx| {
            let item = require_item(&tx, item_id).await?;
            if item.state != RunQueueItemState::Queued {
                return Err(RekallError::conflict("Only a task still waiting its turn can be moved."));
            }
            let all = ordered(&tx).await?;
            let mut waiting: Vec<run_queue_item::Model> = all
                .iter()
                .filter(|candidate| candidate.state == RunQueueItemState::Queued && candidate.id != item.id)
                .cloned()
                .collect();
            let at = index.clamp(0, waiting.len() as i32) as usize;
            waiting.insert(at, item);
            let mut reordered: Vec<run_queue_item::Model> =
                all.iter().filter(|candidate| candidate.state != RunQueueItemState::Queued).cloned().collect();
            reordered.extend(waiting);
            renumber(&tx, reordered).await?;
            let queue = self.queue(&tx).await?;
            self.publish(&mut tx, queue).await
        })
    }

    /// Take every finished, skipped and failed item off the list.
    pub async fn clear_settled(&self) -> Result<RunQueueView> {
        in_write!(&self.ctx, |tx| {
            let all = ordered(&tx).await?;
            let settled: Vec<Id> = all.iter().filter(|item| item.state.settled()).map(|item| item.id).collect();
            if !settled.is_empty() {
                run_queue_item::Entity::delete_many()
                    .filter(run_queue_item::Column::Id.is_in(settled))
                    .exec(tx.db())
                    .await?;
            }
            renumber(&tx, all.into_iter().filter(|item| !item.state.settled()).collect()).await?;
            let queue = self.queue(&tx).await?;
            self.publish(&mut tx, queue).await
        })
    }

    async fn enqueue(&self, tx: &Tx, task_id: Id) -> Result<()> {
        let task = repo::task::find_by_id(tx.db(), task_id)
            .await?
            .ok_or_else(|| RekallError::unknown_anchor(format!("No task with id {task_id}")))?;
        task.require_workable("is never queued to run")?;
        let current = ordered(tx).await?;
        let waiting = current.iter().any(|item| item.task_id == task_id && !item.state.settled());
        if waiting {
            return Err(RekallError::conflict(format!("'{}' is already in the queue.", task.title)));
        }
        run_queue_item::Model::new(task_id, current.len() as i32, self.now())
            .into_active_model()
            .insert(tx.db())
            .await?;
        Ok(())
    }
}
