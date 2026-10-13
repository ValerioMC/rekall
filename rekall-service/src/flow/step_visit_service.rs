use rekall_common::{Id, Result};
use rekall_model::step_visit;
use rekall_repository::repository as repo;
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use crate::{Ctx, Tx};

/// The flow's visits through a task's steps. At most one visit per task is open: entering a step
/// leaves whichever one the flow stood on.
#[derive(Clone)]
pub struct StepVisitService {
    ctx: Ctx,
}

impl StepVisitService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    /// The flow stands on `step_id`. Already standing there keeps that visit; standing elsewhere
    /// leaves it without a route, since the session moved on without claiming it.
    pub async fn enter_in(&self, tx: &Tx, task_id: Id, step_id: Id) -> Result<step_visit::Model> {
        if let Some(open) = repo::step_visit::find_open(tx.db(), task_id).await? {
            if open.step_id == step_id {
                return Ok(open);
            }
            self.close(tx, open, None).await?;
        }
        let ordinal = repo::step_visit::last_ordinal(tx.db(), task_id).await? + 1;
        let visit = step_visit::Model::enter(task_id, step_id, ordinal, self.ctx.now());
        Ok(visit.into_active_model().insert(tx.db()).await?)
    }

    /// Leaves `step_id` by `outcome`. A step claimed without being started is entered first, so
    /// every pass through it is counted.
    pub async fn leave_in(&self, tx: &Tx, task_id: Id, step_id: Id, outcome: Option<String>) -> Result<step_visit::Model> {
        let open = self.enter_in(tx, task_id, step_id).await?;
        self.close(tx, open, outcome).await
    }

    /// Forgets every visit of the task: the next start enters the flow afresh.
    pub async fn clear_in(&self, tx: &Tx, task_id: Id) -> Result<u64> {
        repo::step_visit::delete_of_task(tx.db(), task_id).await
    }

    async fn close(&self, tx: &Tx, visit: step_visit::Model, outcome: Option<String>) -> Result<step_visit::Model> {
        let mut closed = visit;
        closed.leave(outcome, self.ctx.now());
        Ok(closed.into_active_model().reset_all().update(tx.db()).await?)
    }
}
