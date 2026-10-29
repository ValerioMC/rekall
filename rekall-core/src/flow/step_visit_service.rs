use rekall_kernel::{Id, RekallError, Result};
use rekall_store::{step_visit, task_step};
use rekall_store::repository as repo;
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

    /// The flow stands on `step`. Already standing there keeps that visit; standing elsewhere
    /// leaves it without a route, since the session moved on without claiming it. A step whose
    /// visit budget is spent is refused, which rolls back the move that asked.
    pub async fn enter_in(&self, tx: &Tx, step: &task_step::Model) -> Result<step_visit::Model> {
        let task_id = step.task_id;
        if let Some(open) = repo::step_visit::find_open(tx.db(), task_id).await? {
            if open.step_id == step.id {
                return Ok(open);
            }
            self.close(tx, open, None, None).await?;
        }
        let entered = repo::step_visit::count_of_step(tx.db(), step.id).await? as usize;
        refuse_spent_budget(step, entered)?;
        let ordinal = repo::step_visit::last_ordinal(tx.db(), task_id).await? + 1;
        let visit = step_visit::Model::enter(task_id, step.id, ordinal, self.ctx.now());
        Ok(visit.into_active_model().insert(tx.db()).await?)
    }

    /// Leaves `step_id` by `outcome`. A step claimed without being started is entered first, so
    /// every pass through it is counted.
    pub async fn leave_in(&self, tx: &Tx, step: &task_step::Model, outcome: Option<String>, report: Option<String>) -> Result<step_visit::Model> {
        let open = self.enter_in(tx, step).await?;
        self.close(tx, open, outcome, report).await
    }

    /// Forgets every visit of the task: the next start enters the flow afresh.
    pub async fn clear_in(&self, tx: &Tx, task_id: Id) -> Result<u64> {
        repo::step_visit::delete_of_task(tx.db(), task_id).await
    }

    async fn close(&self, tx: &Tx, visit: step_visit::Model, outcome: Option<String>, report: Option<String>) -> Result<step_visit::Model> {
        let mut closed = visit;
        closed.leave(outcome, report, self.ctx.now());
        Ok(closed.into_active_model().reset_all().update(tx.db()).await?)
    }
}

/// A spent budget stops the flow rather than letting an unattended loop go round again. The
/// person decides what next: raise the budget or reset the moves.
fn refuse_spent_budget(step: &task_step::Model, entered: usize) -> Result<()> {
    if step.admits_visit(entered) {
        return Ok(());
    }
    Err(RekallError::illegal(format!(
        "'{}' has been entered {entered} times, its whole visit budget, so the flow stops instead of going round \
         again. Nothing was moved. Stop and tell the person what keeps sending the work back; they can raise the \
         budget or reset the moves in the console.",
        step.title
    )))
}
