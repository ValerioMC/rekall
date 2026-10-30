use rekall_kernel::{Id, RekallError, Result};

use rekall_kernel::jstr;

use crate::flow::{FlowPlan, ProposedFlow, Replanned, RouteTarget, StepLinkService};
use crate::step::{Staging, TaskStepService, TaskStepView};
use crate::{in_write, load, Ctx, Tx};

/// Lays a planned flow on a task as steps and the routes between them, in one transaction. It
/// holds no queries: drafts go through `TaskStepService`, routes through `StepLinkService`.
#[derive(Clone)]
pub struct FlowProposalService {
    ctx: Ctx,
    steps: TaskStepService,
    links: StepLinkService,
}

impl FlowProposalService {
    pub fn new(ctx: Ctx, steps: TaskStepService, links: StepLinkService) -> Self {
        Self { ctx, steps, links }
    }

    /// Only onto a task with no steps: the flow begins at the first one, so a checklist already
    /// there would decide where it starts and end it at its own unrouted steps.
    pub async fn propose(
        &self,
        project_label: Option<&str>,
        task_label: &str,
        plan: &FlowPlan,
        staging: Staging,
    ) -> Result<ProposedFlow> {
        plan.check()?;
        in_write!(&self.ctx, |tx| {
            let task = load::resolve_task(tx.db(), project_label, task_label).await?;
            require_no_steps(&self.steps.find_by_task_in(&mut tx, task.id).await?)?;
            let laid = self.lay_in(&mut tx, task.id, plan).await?;
            if staging == Staging::Ready {
                self.steps.release_drafts_in(&tx, task.id).await?;
            }
            self.steps.publish_in(&mut tx, task.id).await?;
            Ok(laid)
        })
    }

    /// Replaces the task's draft and open steps with the plan, in one transaction, after the plan is
    /// checked whole. Started, claimed and done steps stay: the plan follows them, and a flow cannot,
    /// since it begins at the first step. A title one of them has is refused, and nothing is deleted.
    pub async fn replan(
        &self,
        project_label: Option<&str>,
        task_label: &str,
        plan: &FlowPlan,
        staging: Staging,
    ) -> Result<Replanned> {
        plan.check_as_written()?;
        in_write!(&self.ctx, |tx| {
            let task = load::resolve_task(tx.db(), project_label, task_label).await?;
            let kept: Vec<TaskStepView> =
                self.steps.find_by_task_in(&mut tx, task.id).await?.into_iter().filter(|step| !step.state.unstarted()).collect();
            require_flow_has_no_kept_steps(plan, &kept)?;
            require_titles_free_of(plan, &kept)?;
            let removed = self.steps.clear_unstarted_in(&tx, task.id).await?;
            let laid = self.lay_in(&mut tx, task.id, plan).await?;
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
            let draft = self.steps.add_in(tx, task_id, Some(&planned.title), planned.detail.as_deref()).await?;
            if planned.max_visits.is_some() {
                self.steps.limit_visits_in(tx, draft.id, planned.max_visits).await?;
            }
            if planned.starts_session {
                self.steps.begin_session_in(tx, draft.id, true).await?;
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

fn require_no_steps(existing: &[TaskStepView]) -> Result<()> {
    if existing.is_empty() {
        return Ok(());
    }
    Err(RekallError::illegal(format!(
        "A flow is proposed whole, onto a task with no steps, and this one has {}. Propose single steps with \
         `rekall_propose_step`, or clear the steps in the console first. Nothing was proposed.",
        existing.len()
    )))
}

fn require_flow_has_no_kept_steps(plan: &FlowPlan, kept: &[TaskStepView]) -> Result<()> {
    if plan.route_count() == 0 || kept.is_empty() {
        return Ok(());
    }
    Err(RekallError::illegal(format!(
        "A flow begins at the first step, and {} stay on this task because they were started or finished: {}. \
         Replan as a checklist, which follows them, or let a person clear them in the console. Nothing was changed.",
        kept.len(),
        titles_of(kept)
    )))
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

fn titles_of(steps: &[TaskStepView]) -> String {
    steps.iter().map(|step| format!("'{}' ({})", step.title, step.state.name().to_lowercase())).collect::<Vec<_>>().join(", ")
}
