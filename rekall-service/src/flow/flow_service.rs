use rekall_common::{Id, RekallError, Result};
use rekall_model::{step_link, task_step, TaskStepState};

use super::{choose_route, FlowMove, FlowRoute, StepLinkService, StepVisitService};
use crate::step::{TaskStepService, TaskStepView};
use crate::{in_write, load, Ctx, Tx};

/// A session's move through a task's steps, routed when the task is a flow: a start enters the
/// step, a claim leaves it by its outcome and opens the step that outcome leads to. On a plain
/// checklist it is the step move and nothing else.
#[derive(Clone)]
pub struct FlowService {
    ctx: Ctx,
    steps: TaskStepService,
    links: StepLinkService,
    visits: StepVisitService,
}

impl FlowService {
    pub fn new(ctx: Ctx, steps: TaskStepService, links: StepLinkService, visits: StepVisitService) -> Self {
        Self { ctx, steps, links, visits }
    }

    pub async fn transition(
        &self,
        project_label: Option<&str>,
        task_label: &str,
        step_ref: Option<&str>,
        target: TaskStepState,
        outcome: Option<&str>,
    ) -> Result<FlowMove> {
        in_write!(&self.ctx, |tx| {
            let task = load::resolve_task(tx.db(), project_label, task_label).await?;
            let is_flow = self.links.is_flow_in(&tx, task.id).await?;
            let outcome = outcome_for(is_flow, target, outcome)?;
            let moved = self.steps.transition_in(&tx, &task, step_ref, target).await?;
            let route = match (is_flow, target) {
                (true, TaskStepState::Running) => {
                    self.visits.enter_in(&tx, task.id, moved.id).await?;
                    None
                }
                (true, TaskStepState::Claimed) => Some(self.leave_by(&tx, &moved, outcome.as_deref()).await?),
                _ => None,
            };
            self.steps.publish_in(&mut tx, task.id).await?;
            Ok(FlowMove { step: self.steps.view_in(&tx, &moved).await?, route })
        })
    }

    /// Forgets where the flow went: every visit is dropped and the counts start from zero. The
    /// steps keep their states, since those are the person's review.
    pub async fn reset(&self, task_id: Id) -> Result<Vec<TaskStepView>> {
        in_write!(&self.ctx, |tx| {
            load::task_or_unknown(tx.db(), task_id).await?;
            self.visits.clear_in(&tx, task_id).await?;
            self.steps.publish_in(&mut tx, task_id).await?;
            self.steps.find_by_task_in(&mut tx, task_id).await
        })
    }

    /// Closes the visit on `claimed` by the route its outcome names and enters that route's step.
    async fn leave_by(&self, tx: &Tx, claimed: &task_step::Model, outcome: Option<&str>) -> Result<FlowRoute> {
        let outgoing = self.links.routes_from_in(tx, claimed.id).await?;
        let Some(link) = choose_route(&claimed.title, &outgoing, outcome)? else {
            self.visits.leave_in(tx, claimed.task_id, claimed.id, None).await?;
            return Ok(FlowRoute::Ended { outcome: None });
        };
        self.visits.leave_in(tx, claimed.task_id, claimed.id, Some(link.outcome.clone())).await?;
        let Some(next_step_id) = link.to_step_id else {
            return Ok(FlowRoute::Ended { outcome: Some(link.outcome.clone()) });
        };
        Ok(FlowRoute::Followed { outcome: link.outcome.clone(), next: Box::new(self.enter_step(tx, next_step_id).await?) })
    }

    async fn enter_step(&self, tx: &Tx, step_id: Id) -> Result<TaskStepView> {
        let next = self.steps.reopen_for_route_in(tx, step_id).await?;
        self.visits.enter_in(tx, next.task_id, next.id).await?;
        self.steps.view_in(tx, &next).await
    }
}

/// The outcome to route by, normalised. It belongs to a claim on a flow; anywhere else it would
/// be silently ignored, so it is refused.
fn outcome_for(is_flow: bool, target: TaskStepState, outcome: Option<&str>) -> Result<Option<String>> {
    let Some(raw) = outcome.filter(|raw| !raw.trim().is_empty()) else {
        return Ok(None);
    };
    if !is_flow {
        return Err(RekallError::illegal(
            "This task's steps carry no routes, so there is no outcome to follow. Claim without one.",
        ));
    }
    if target != TaskStepState::Claimed {
        return Err(RekallError::illegal("An outcome is passed with `claimed`, when the step ends. Nothing was moved."));
    }
    step_link::normalise_outcome(Some(raw)).map(Some)
}

#[cfg(test)]
#[path = "../../tests/unit/flow/flow_service_tests.rs"]
mod tests;
