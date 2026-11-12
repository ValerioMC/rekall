//! Where a step sits on the flow canvas and how many times a flow may enter it.

use rekall_kernel::{Id, Result};
use rekall_store::repository as repo;
use rekall_store::task_step;

use crate::step::{FlowPlace, TaskStepService, TaskStepView, VisitBudget};
use crate::{in_write, load, Ctx, Tx};

/// Writes only the canvas place and the visit cap of a step, on top of the checklist
/// `TaskStepService` keeps.
#[derive(Clone)]
pub struct StepPlacementService {
    ctx: Ctx,
    steps: TaskStepService,
}

impl StepPlacementService {
    pub fn new(ctx: Ctx, steps: TaskStepService) -> Self {
        Self { ctx, steps }
    }

    /// Remembers where the person dragged a step on the flow canvas. `None` hands it back to the
    /// automatic layout. Answers the task's whole checklist, like a route does.
    pub async fn place_in_flow(&self, id: Id, place: Option<FlowPlace>) -> Result<Vec<TaskStepView>> {
        in_write!(&self.ctx, |tx| {
            let before = repo::task_step::require(tx.db(), id).await?;
            let mut step = before.clone();
            step.flow_x = place.map(|at| at.x);
            step.flow_y = place.map(|at| at.y);
            repo::update_if_changed(tx.db(), &before, step, self.ctx.now()).await?;
            self.steps.publish_in(&mut tx, before.task_id).await?;
            self.steps.find_by_task_in(&mut tx, before.task_id).await
        })
    }

    /// Caps how many times a flow may enter the step; `None` lifts the cap. Answers the task's
    /// whole checklist, like a route does.
    pub async fn limit_visits(&self, id: Id, budget: Option<VisitBudget>) -> Result<Vec<TaskStepView>> {
        in_write!(&self.ctx, |tx| {
            let saved = self.limit_visits_in(&tx, id, budget).await?;
            self.steps.publish_in(&mut tx, saved.task_id).await?;
            self.steps.find_by_task_in(&mut tx, saved.task_id).await
        })
    }

    pub async fn limit_visits_in(&self, tx: &Tx, id: Id, budget: Option<VisitBudget>) -> Result<task_step::Model> {
        let before = repo::task_step::require(tx.db(), id).await?;
        let mut step = before.clone();
        step.max_visits = budget.map(VisitBudget::visits);
        repo::update_if_changed(tx.db(), &before, step, self.ctx.now()).await
    }

    /// Hands every step of the task back to the automatic layout.
    pub async fn clear_flow_places(&self, task_id: Id) -> Result<Vec<TaskStepView>> {
        in_write!(&self.ctx, |tx| {
            load::task_or_unknown(tx.db(), task_id).await?;
            for before in load::steps_of(tx.db(), task_id).await? {
                let mut step = before.clone();
                step.flow_x = None;
                step.flow_y = None;
                repo::update_if_changed(tx.db(), &before, step, self.ctx.now()).await?;
            }
            self.steps.publish_in(&mut tx, task_id).await?;
            self.steps.find_by_task_in(&mut tx, task_id).await
        })
    }
}
