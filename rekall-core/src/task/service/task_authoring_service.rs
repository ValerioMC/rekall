use rekall_kernel::{RekallError, Result};
use rekall_store::task::Model;
use rekall_store::TaskKind;
use rekall_store::repository as repo;
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use crate::flow::{FlowPlan, FlowProposalService};
use crate::task::{Created, TaskBrief, TaskCreatedEvent, TaskStreamEvent};
use crate::step::TaskStepService;
use crate::{in_write, load, text_rule, Ctx};

/// Creates a task or an epic for a session. The task starts `TODO` and its steps, a checklist or a
/// flow with its routes and sessions, start open: the session wrote them to be worked, not reviewed.
#[derive(Clone)]
pub struct TaskAuthoringService {
    ctx: Ctx,
    steps: TaskStepService,
    flows: FlowProposalService,
}

impl TaskAuthoringService {
    pub fn new(ctx: Ctx, steps: TaskStepService, flows: FlowProposalService) -> Self {
        Self { ctx, steps, flows }
    }

    /// The task takes the next code of its project; `brief.epic` files the task under an epic of the same
    /// project. A plan with a route is checked as a flow, one without as a checklist. Task, steps and
    /// routes land in one transaction, so a refused plan leaves no half-made task behind.
    pub async fn create(&self, project_label: &str, brief: &TaskBrief, plan: &FlowPlan) -> Result<Created> {
        require_plan_fits(plan)?;
        require_epic_stepless(brief.kind, plan)?;
        in_write!(&self.ctx, |tx| {
            let project = load::resolve_project(tx.db(), project_label).await?;
            let title = required_title(brief.title.as_deref())?;
            let mut task = Model::new(String::new(), title, project.id, self.ctx.now());
            repo::task::assign_next_code(tx.db(), &mut task, &project).await?;
            task.description = text_rule::present(brief.description.as_deref()).map(str::to_string);
            task.kind = brief.kind;
            if let Some(epic_label) = brief.epic.as_deref() {
                let epic = load::resolve_task(tx.db(), Some(&project.label), epic_label).await?;
                task.require_joinable(&epic)?;
                task.epic_id = Some(epic.id);
            }
            task.validate()?;
            task.clone().into_active_model().insert(tx.db()).await?;
            tx.publish(TaskStreamEvent { task_id: task.id, project_id: project.id });
            tx.publish(TaskCreatedEvent { task_id: task.id });
            let laid = self.flows.lay_in(&mut tx, task.id, plan).await?;
            self.steps.release_drafts_in(&tx, task.id).await?;
            self.steps.publish_in(&mut tx, task.id).await?;
            Ok(Created { task_anchor: task.anchor(), title: task.title, kind: task.kind, steps: laid.steps, routes: laid.routes })
        })
    }
}

fn required_title(title: Option<&str>) -> Result<String> {
    text_rule::present(title).map(str::to_string).ok_or_else(|| RekallError::illegal("A task needs a title."))
}

fn require_plan_fits(plan: &FlowPlan) -> Result<()> {
    if plan.steps.is_empty() {
        return Ok(());
    }
    plan.check_as_written()
}

/// Refused before anything is written, so the answer names the steps rather than the first one.
fn require_epic_stepless(kind: TaskKind, plan: &FlowPlan) -> Result<()> {
    if kind == TaskKind::Epic && !plan.steps.is_empty() {
        return Err(RekallError::illegal(
            "An epic holds no steps. Nothing was created: create the epic without `steps`, then create its tasks with `epic` set to it.",
        ));
    }
    Ok(())
}
