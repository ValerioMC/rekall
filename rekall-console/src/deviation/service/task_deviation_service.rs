use rekall_core::deviation::step_at;
use rekall_core::{in_read, Ctx};
use rekall_kernel::{Id, Result};
use rekall_store::repository as repo;

use crate::deviation::TaskDeviationView;

#[derive(Clone)]
pub struct TaskDeviationService {
    ctx: Ctx,
}

impl TaskDeviationService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn list_of(&self, task_id: Id) -> Result<Vec<TaskDeviationView>> {
        in_read!(&self.ctx, |tx| {
            repo::task::require(tx.db(), task_id).await?;
            let steps = repo::task_step::list_of_task(tx.db(), task_id).await?;
            let deviations = repo::task_deviation::list_of_task(tx.db(), task_id).await?;
            Ok(deviations
                .iter()
                .map(|deviation| TaskDeviationView::of(deviation, step_at(&steps, deviation.step_number).map(|step| step.title.clone())))
                .collect())
        })
    }
}
