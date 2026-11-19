use rekall_kernel::{Id, RekallError, Result};

use rekall_kernel::jstr;

use crate::flow::{FlowPlan, ProposedFlow, Replanned, RouteTarget, StepLinkService, StepPlacementService};
use crate::step::{NewStep, Staging, StepBindingService, TaskStepService, TaskStepView};
use crate::{in_write, load, Ctx, Tx};

/// Lays a planned flow on a task as steps and the routes between them, in one transaction. It
/// holds no queries: drafts go through `TaskStepService`, routes through `StepLinkService`.
#[derive(Clone)]
pub struct FlowProposalService {
    ctx: Ctx,
    steps: TaskStepService,
    links: StepLinkService,
    placement: StepPlacementService,
    bindings: StepBindingService,
}

impl FlowProposalService {
    pub fn new(ctx: Ctx, steps: TaskStepService, links: StepLinkService, placement: StepPlacementService, bindings: StepBindingService) -> Self {
        Self { ctx, steps, links, placement, bindings }
    }

    /// Only onto a task with no steps, or whose steps are all finished: they stay as history and
    /// the flow begins at the plan's first step. An open step already there would decide where it
    /// starts and end it at its own unrouted steps.
    pub async fn propose(
        &self,
        project_label: Option<&str>,
        task_label: &str,
        plan: &FlowPlan,
        staging: Staging,
    ) -> Result<ProposedFlow> {
        in_write!(&self.ctx, |tx| {
            let task = load::resolve_task(tx.db(), project_label, task_label).await?;
            let existing = self.steps.find_by_task_in(&mut tx, task.id).await?;
            require_only_finished_steps(&existing)?;
            let plan = following(plan, &existing)?;
            plan.check()?;
            let laid = self.lay_in(&mut tx, task.id, &plan).await?;
            if staging == Staging::Ready {
                self.steps.release_drafts_in(&tx, task.id).await?;
            }
            self.steps.publish_in(&mut tx, task.id).await?;
            Ok(laid)
        })
    }

    /// Replaces the task's draft and open steps with the plan, in one transaction, after the plan is
    /// checked whole. Started, claimed and done steps stay and the plan follows them; a flow is laid
    /// after them only once none is still running. A title one of them has is refused, and nothing
    /// is deleted.
    pub async fn replan(
        &self,
        project_label: Option<&str>,
        task_label: &str,
        plan: &FlowPlan,
        staging: Staging,
    ) -> Result<Replanned> {
        in_write!(&self.ctx, |tx| {
            let task = load::resolve_task(tx.db(), project_label, task_label).await?;
            let kept: Vec<TaskStepView> =
                self.steps.find_by_task_in(&mut tx, task.id).await?.into_iter().filter(|step| !step.state.unstarted()).collect();
            require_flow_follows_finished_steps(plan, &kept)?;
            let plan = following(plan, &kept)?;
            plan.check_as_written()?;
            require_titles_free_of(&plan, &kept)?;
            let removed = self.steps.clear_unstarted_in(&tx, task.id).await?;
            let laid = self.lay_in(&mut tx, task.id, &plan).await?;
            if staging == Staging::Ready {
                self.steps.release_drafts_in(&tx, task.id).await?;
            }
            self.steps.publish_in(&mut tx, task.id).await?;
            Ok(Replanned { removed, laid, kept: kept.len() })
        })
    }

    /// The plan's drafts, sessions, visit budgets and routes, inside a transaction the caller holds.
    /// The caller has checked the plan and publishes.
    pub async fn lay_in(&self, tx: &mut Tx, task_id: Id, plan: &FlowPlan) -> Result<ProposedFlow> {
        let mut drafts = Vec::with_capacity(plan.steps.len());
        for planned in &plan.steps {
            let draft = self.steps.add_in(tx, task_id, NewStep::titled(&planned.title, planned.detail.as_deref())).await?;
            if planned.max_visits.is_some() {
                self.placement.limit_visits_in(tx, draft.id, planned.max_visits).await?;
            }
            if planned.starts_session {
                self.bindings.begin_session_in(tx, draft.id, true).await?;
            }
            drafts.push(draft);
        }
        for (planned, draft) in plan.steps.iter().zip(&drafts) {
            for route in &planned.routes {
                let target = route.to.map_or(RouteTarget::End, |to| {
                    RouteTarget::Step(drafts.get(to).expect("FlowPlan::check keeps every route inside the plan").id)
                });
                self.links.propose_in(tx, draft.id, target, &route.outcome).await?;
            }
        }
        Ok(ProposedFlow { steps: drafts.len(), routes: plan.route_count() })
    }
}

fn require_only_finished_steps(existing: &[TaskStepView]) -> Result<()> {
    if existing.iter().all(|step| step.state.complete()) {
        return Ok(());
    }
    if let Some(running) = existing.iter().find(|step| step.state.running()) {
        return Err(running_step_refusal(running));
    }
    Err(RekallError::illegal(format!(
        "A flow is proposed whole, onto a task with no steps or only finished ones, and this one has {} that are not. \
         Propose single steps with `rekall_propose_step`, replan it with `rekall_replan`, or clear the steps in the \
         console first. Nothing was proposed.",
        existing.iter().filter(|step| !step.state.complete()).count()
    )))
}

fn require_flow_follows_finished_steps(plan: &FlowPlan, kept: &[TaskStepView]) -> Result<()> {
    if plan.route_count() == 0 {
        return Ok(());
    }
    match kept.iter().find(|step| step.state.running()) {
        Some(running) => Err(running_step_refusal(running)),
        None => Ok(()),
    }
}

fn running_step_refusal(running: &TaskStepView) -> RekallError {
    RekallError::illegal(format!(
        "'{}' is still running, so how it ends is not known yet and the flow cannot be laid after it. Claim it with \
         `rekall_step` first, then plan the flow. Nothing was changed.",
        running.title
    ))
}

fn following(plan: &FlowPlan, kept: &[TaskStepView]) -> Result<FlowPlan> {
    let titles: Vec<&str> = kept.iter().map(|step| step.title.as_str()).collect();
    plan.following(&titles)
}

fn require_titles_free_of(plan: &FlowPlan, kept: &[TaskStepView]) -> Result<()> {
    for planned in &plan.steps {
        let taken = kept.iter().find(|step| jstr::equals_ignore_case(jstr::strip(&step.title), jstr::strip(&planned.title)));
        if let Some(step) = taken {
            return Err(RekallError::illegal(format!(
                "This task already has a step titled '{}' ({}) that stays. Nothing was changed.",
                step.title,
                step.state.name().to_lowercase()
            )));
        }
    }
    Ok(())
}
