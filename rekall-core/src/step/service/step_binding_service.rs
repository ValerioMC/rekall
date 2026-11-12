//! What a step is tied to besides its text: the session it opens and the connection it reaches.

use rekall_kernel::{Id, Result};
use rekall_store::repository as repo;
use rekall_store::task_step;

use crate::step::{TaskStepService, TaskStepView};
use crate::{in_write, Ctx, Tx};

/// Writes only the two bindings of a step, on top of the checklist `TaskStepService` keeps.
#[derive(Clone)]
pub struct StepBindingService {
    ctx: Ctx,
    steps: TaskStepService,
}

impl StepBindingService {
    pub fn new(ctx: Ctx, steps: TaskStepService) -> Self {
        Self { ctx, steps }
    }

    /// Makes the step open a fresh session, or join the one running. Answers the task's whole
    /// checklist, because the sessions group the steps around it.
    pub async fn begin_session(&self, id: Id, starts_session: bool) -> Result<Vec<TaskStepView>> {
        in_write!(&self.ctx, |tx| {
            let saved = self.begin_session_in(&tx, id, starts_session).await?;
            self.steps.publish_in(&mut tx, saved.task_id).await?;
            self.steps.find_by_task_in(&mut tx, saved.task_id).await
        })
    }

    pub async fn begin_session_in(&self, tx: &Tx, id: Id, starts_session: bool) -> Result<task_step::Model> {
        let before = repo::task_step::require(tx.db(), id).await?;
        let step = task_step::Model { starts_session, ..before.clone() };
        repo::update_if_changed(tx.db(), &before, step, self.ctx.now()).await
    }

    /// Sets the connection the step reaches, as a deployment graph laid on the task says.
    pub async fn reach_connection_in(&self, tx: &Tx, id: Id, connection_id: Option<Id>) -> Result<task_step::Model> {
        let before = repo::task_step::require(tx.db(), id).await?;
        let step = task_step::Model { connection_id, ..before.clone() };
        repo::update_if_changed(tx.db(), &before, step, self.ctx.now()).await
    }
}
