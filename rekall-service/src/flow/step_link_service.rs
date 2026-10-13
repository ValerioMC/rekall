use rekall_common::{Id, RekallError, Result};
use rekall_model::step_link::{self, ROUTES_PER_STEP_MAX};
use rekall_model::task_step;
use rekall_repository::repository as repo;
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use crate::step::{TaskStepService, TaskStepView};
use crate::{in_write, Ctx, Tx};

/// The routes between a task's steps. Written from the console only; a session reads them in the
/// context and follows them through `FlowService`, never edits them.
#[derive(Clone)]
pub struct StepLinkService {
    ctx: Ctx,
    steps: TaskStepService,
}

impl StepLinkService {
    pub fn new(ctx: Ctx, steps: TaskStepService) -> Self {
        Self { ctx, steps }
    }

    /// A route from `from_step_id` to `to_step_id` when the first ends with `outcome`. Answers
    /// the task's whole checklist, since a route changes how the flow reads.
    pub async fn add(&self, from_step_id: Id, to_step_id: Id, outcome: Option<&str>) -> Result<Vec<TaskStepView>> {
        in_write!(&self.ctx, |tx| {
            let (from, to) = self.endpoints(&tx, from_step_id, to_step_id).await?;
            let outcome = step_link::normalise_outcome(outcome)?;
            let outgoing = repo::step_link::list_from(tx.db(), from.id).await?;
            if outgoing.len() >= ROUTES_PER_STEP_MAX {
                return Err(RekallError::illegal(format!(
                    "'{}' already has {ROUTES_PER_STEP_MAX} routes out, the most one step can decide between. Split it.",
                    from.title
                )));
            }
            refuse_taken_outcome(&from, &outgoing, &outcome, None)?;
            let link = step_link::Model::new(from.task_id, from.id, to.id, outcome, outgoing.len() as i32, self.ctx.now());
            link.validate()?;
            link.into_active_model().insert(tx.db()).await?;
            self.answer(&mut tx, from.task_id).await
        })
    }

    /// Points a route at another step, renames its outcome, or both.
    pub async fn edit(&self, id: Id, to_step_id: Option<Id>, outcome: Option<&str>) -> Result<Vec<TaskStepView>> {
        in_write!(&self.ctx, |tx| {
            let before = repo::step_link::require(tx.db(), id).await?;
            let mut link = before.clone();
            if let Some(to_step_id) = to_step_id {
                link.to_step_id = self.endpoints(&tx, before.from_step_id, to_step_id).await?.1.id;
            }
            if outcome.is_some() {
                link.outcome = step_link::normalise_outcome(outcome)?;
                let from = repo::task_step::require(tx.db(), before.from_step_id).await?;
                let outgoing = repo::step_link::list_from(tx.db(), from.id).await?;
                refuse_taken_outcome(&from, &outgoing, &link.outcome, Some(id))?;
            }
            repo::update_if_changed(tx.db(), &before, link, self.ctx.now()).await?;
            self.answer(&mut tx, before.task_id).await
        })
    }

    pub async fn delete(&self, id: Id) -> Result<Vec<TaskStepView>> {
        in_write!(&self.ctx, |tx| {
            let link = repo::step_link::require(tx.db(), id).await?;
            repo::step_link::delete(tx.db(), id).await?;
            self.answer(&mut tx, link.task_id).await
        })
    }

    /// The routes leaving a step, in the order the person ranked them.
    pub async fn routes_from_in(&self, tx: &Tx, step_id: Id) -> Result<Vec<step_link::Model>> {
        repo::step_link::list_from(tx.db(), step_id).await
    }

    /// Whether any of the task's steps carries a route, which is what makes its checklist a flow.
    pub async fn is_flow_in(&self, tx: &Tx, task_id: Id) -> Result<bool> {
        Ok(!repo::step_link::list_of_task(tx.db(), task_id).await?.is_empty())
    }

    /// Both ends of a route: two steps of one task.
    async fn endpoints(&self, tx: &Tx, from_step_id: Id, to_step_id: Id) -> Result<(task_step::Model, task_step::Model)> {
        let from = repo::task_step::require(tx.db(), from_step_id).await?;
        let to = repo::task_step::require(tx.db(), to_step_id).await?;
        if from.task_id != to.task_id {
            return Err(RekallError::illegal(format!(
                "'{}' and '{}' are on different tasks. A route joins two steps of one task.",
                from.title, to.title
            )));
        }
        Ok((from, to))
    }

    async fn answer(&self, tx: &mut Tx, task_id: Id) -> Result<Vec<TaskStepView>> {
        self.steps.publish_in(tx, task_id).await?;
        self.steps.find_by_task_in(tx, task_id).await
    }
}

/// One outcome names one route out of a step, or a session could not say which it took.
fn refuse_taken_outcome(from: &task_step::Model, outgoing: &[step_link::Model], outcome: &str, except: Option<Id>) -> Result<()> {
    let taken = outgoing.iter().any(|link| link.outcome == outcome && Some(link.id) != except);
    if taken {
        return Err(RekallError::conflict(format!("'{}' already routes by `{outcome}`. Pick another outcome.", from.title)));
    }
    Ok(())
}
