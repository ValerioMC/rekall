use rekall_common::{RekallError, Result};

use super::{FlowPlan, ProposedFlow, RouteTarget, StepLinkService};
use crate::step::{TaskStepService, TaskStepView};
use crate::{in_write, load, Ctx};

/// Lays a planned flow on a task as drafts and the routes between them, in one transaction. It
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
    pub async fn propose(&self, project_label: Option<&str>, task_label: &str, plan: &FlowPlan) -> Result<ProposedFlow> {
        plan.check()?;
        in_write!(&self.ctx, |tx| {
            let task = load::resolve_task(tx.db(), project_label, task_label).await?;
            require_no_steps(&self.steps.find_by_task_in(&mut tx, task.id).await?)?;
            let mut drafts = Vec::with_capacity(plan.steps.len());
            for planned in &plan.steps {
                let draft = self.steps.add_in(&mut tx, task.id, Some(&planned.title), planned.detail.as_deref()).await?;
                if planned.max_visits.is_some() {
                    self.steps.limit_visits_in(&tx, draft.id, planned.max_visits).await?;
                }
                drafts.push(draft);
            }
            for (planned, draft) in plan.steps.iter().zip(&drafts) {
                for route in &planned.routes {
                    let target = route.to.map_or(RouteTarget::End, |to| {
                        RouteTarget::Step(drafts.get(to).expect("FlowPlan::check keeps every route inside the plan").id)
                    });
                    self.links.propose_in(&tx, draft.id, target, &route.outcome).await?;
                }
            }
            self.steps.publish_in(&mut tx, task.id).await?;
            Ok(ProposedFlow { steps: drafts.len(), routes: plan.route_count() })
        })
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
