use rekall_common::{jstr, Id, RekallError, Result};
use rekall_model::task_step::{self, MAX_CHARACTERS, TITLE_MAX_CHARACTERS};
use rekall_model::TaskStepState;
use rekall_repository::repository as repo;
use rekall_model::task;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use super::{FlowPlace, StepFlowRows, StepStreamEvent, TaskStepView, VisitBudget};
use crate::timeentry::TimeEntryService;
use crate::{in_read, in_write, load, text_rule, Ctx, Tx};

use super::Proposed;

/// Drafts a session may leave on one task before a person has promoted or deleted some.
pub const PROPOSED_DRAFTS_MAX: usize = 20;

#[derive(Clone)]
pub struct TaskStepService {
    ctx: Ctx,
    time_entries: TimeEntryService,
}

impl TaskStepService {
    pub fn new(ctx: Ctx, time_entries: TimeEntryService) -> Self {
        Self { ctx, time_entries }
    }

    pub async fn find_all(&self) -> Result<Vec<TaskStepView>> {
        in_read!(&self.ctx, |tx| {
            let steps = repo::task_step::list_all_in_order(tx.db()).await?;
            let flow = StepFlowRows::of_every_task(tx.db()).await?;
            steps.iter().map(|step| TaskStepView::of(step, &flow)).collect::<Result<Vec<_>>>()
        })
    }

    pub async fn find_by_task(&self, task_id: Id) -> Result<Vec<TaskStepView>> {
        in_read!(&self.ctx, |tx| self.find_by_task_in(&mut tx, task_id).await)
    }

    pub async fn find_by_task_in(&self, tx: &mut Tx, task_id: Id) -> Result<Vec<TaskStepView>> {
        let steps = load::steps_of(tx.db(), task_id).await?;
        let flow = StepFlowRows::of_task(tx.db(), task_id).await?;
        steps.iter().map(|step| TaskStepView::of(step, &flow)).collect()
    }

    pub async fn add(&self, task_id: Id, title: Option<&str>, body_markdown: Option<&str>) -> Result<TaskStepView> {
        in_write!(&self.ctx, |tx| self.add_in(&mut tx, task_id, title, body_markdown).await)
    }

    pub async fn add_in(
        &self,
        tx: &mut Tx,
        task_id: Id,
        title: Option<&str>,
        body_markdown: Option<&str>,
    ) -> Result<TaskStepView> {
        self.insert_in(tx, task_id, title, body_markdown, TaskStepState::Draft).await
    }

    /// Adds a step already promoted to work, for a task the console starts a session on as soon
    /// as it exists: a draft there would leave that session with nothing it may work.
    pub async fn add_open_in(
        &self,
        tx: &mut Tx,
        task_id: Id,
        title: Option<&str>,
        body_markdown: Option<&str>,
    ) -> Result<TaskStepView> {
        self.insert_in(tx, task_id, title, body_markdown, TaskStepState::Open).await
    }

    async fn insert_in(
        &self,
        tx: &mut Tx,
        task_id: Id,
        title: Option<&str>,
        body_markdown: Option<&str>,
        state: TaskStepState,
    ) -> Result<TaskStepView> {
        let task = load::task_or_unknown(tx.db(), task_id).await?;
        require_not_epic(&task)?;
        let siblings = load::steps_of(tx.db(), task_id).await?;
        let mut step = task_step::Model::new(task_id, validated_title(title)?, siblings.len() as i32, self.ctx.now());
        step.mark_state(state);
        step.body_markdown = validated_body(body_markdown)?;
        step.validate()?;
        let saved = step.into_active_model().insert(tx.db()).await?;
        let view = self.view_in(tx, &saved).await?;
        self.publish_in(tx, task_id).await?;
        Ok(view)
    }

    /// Adds a step a session proposes, as a draft at the tail. A draft is not work: it stays off
    /// the checklist a session reads until a person promotes it in the console, which is what
    /// lets a session write one at all. A title the task already has, in any state, is refused,
    /// so a plan run twice does not double the shelf; so is a draft past the cap.
    pub async fn propose(
        &self,
        project_label: Option<&str>,
        task_label: &str,
        title: Option<&str>,
        body_markdown: Option<&str>,
    ) -> Result<Proposed> {
        in_write!(&self.ctx, |tx| {
            let task = load::resolve_task(tx.db(), project_label, task_label).await?;
            let wanted = validated_title(title)?;
            let siblings = load::steps_of(tx.db(), task.id).await?;
            if let Some(existing) = siblings
                .iter()
                .find(|step| jstr::equals_ignore_case(jstr::strip(&step.title), &wanted))
            {
                return Err(RekallError::illegal(format!(
                    "This task already has a step titled '{}' ({}). Nothing was added.",
                    existing.title,
                    existing.state.name().to_lowercase()
                )));
            }
            let drafts = siblings.iter().filter(|step| step.state == TaskStepState::Draft).count();
            if drafts >= PROPOSED_DRAFTS_MAX {
                return Err(RekallError::illegal(format!(
                    "This task already holds {drafts} drafts, the most a session may leave. A person has to promote \
                     or delete some in the console first."
                )));
            }
            let added = self.add_in(&mut tx, task.id, Some(&wanted), body_markdown).await?;
            Ok(Proposed { step: added, draft_count: drafts + 1 })
        })
    }

    /// A console edit. Ticking a claimed step done accepts the claim, which also stops the task's
    /// timer; ticking an open step by hand leaves it running.
    pub async fn edit(
        &self,
        id: Id,
        title: Option<&str>,
        body_markdown: Option<&str>,
        done: Option<bool>,
        draft: Option<bool>,
    ) -> Result<TaskStepView> {
        in_write!(&self.ctx, |tx| {
            let before = self.require(&tx, id).await?;
            let mut step = before.clone();
            if title.is_some() {
                step.title = validated_title(title)?;
            }
            if body_markdown.is_some() {
                step.body_markdown = validated_body(body_markdown)?;
            }
            if let Some(draft) = draft {
                apply_draft(&mut step, draft)?;
            }
            if let Some(done) = done {
                step.mark_state(if done { TaskStepState::Done } else { TaskStepState::Open });
            }
            let task_id = step.task_id;
            let accepts_claim = before.state == TaskStepState::Claimed && step.state == TaskStepState::Done;
            repo::update_if_changed(tx.db(), &before, step, self.ctx.now()).await?;
            if accepts_claim {
                self.time_entries.stop_if_running_in(&mut tx, task_id).await?;
            }
            if draft.is_some() {
                let ordered = load::steps_of(tx.db(), task_id).await?;
                self.settle_drafts_at_tail(&tx, ordered).await?;
            }
            let saved = self.view_in(&tx, &self.require(&tx, id).await?).await?;
            self.publish_in(&mut tx, task_id).await?;
            Ok(saved)
        })
    }

    /// Sends a claimed step back for another pass: what it was worked from is kept as a pass and
    /// the detail is left empty for the feedback. Only the console does this; no MCP tool can.
    pub async fn send_back(&self, id: Id) -> Result<TaskStepView> {
        in_write!(&self.ctx, |tx| {
            let before = self.require(&tx, id).await?;
            let mut step = before.clone();
            step.send_back(self.ctx.now())?;
            let saved = repo::update_if_changed(tx.db(), &before, step, self.ctx.now()).await?;
            let view = self.view_in(&tx, &saved).await?;
            self.publish_in(&mut tx, saved.task_id).await?;
            Ok(view)
        })
    }

    pub async fn transition(
        &self,
        project_label: Option<&str>,
        task_label: &str,
        step_ref: Option<&str>,
        target: TaskStepState,
    ) -> Result<TaskStepView> {
        in_write!(&self.ctx, |tx| {
            let task = load::resolve_task(tx.db(), project_label, task_label).await?;
            let saved = self.transition_in(&tx, &task, step_ref, target).await?;
            let view = self.view_in(&tx, &saved).await?;
            self.publish_in(&mut tx, task.id).await?;
            Ok(view)
        })
    }

    /// The move a session asks for, without announcing it: the caller publishes once its own
    /// writes are in. `DONE` and drafts are refused, and so is a step already accepted.
    pub async fn transition_in(
        &self,
        tx: &Tx,
        task: &task::Model,
        step_ref: Option<&str>,
        target: TaskStepState,
    ) -> Result<task_step::Model> {
        if target == TaskStepState::Done {
            return Err(RekallError::illegal(
                "A step is marked done in the console, by the person who reviewed the work. \
                 A session can take it as far as `claimed`.",
            ));
        }
        let before = load::resolve_step(tx.db(), task.id, step_ref).await?;
        if before.state == TaskStepState::Draft {
            return Err(RekallError::illegal(format!(
                "'{}' is still a draft. It is promoted to a workable step in the console, not from a session.",
                before.title
            )));
        }
        if before.state == TaskStepState::Done {
            return Err(RekallError::illegal(format!(
                "'{}' is already done. Reopen it in the console if that was wrong.",
                before.title
            )));
        }
        let mut step = before.clone();
        step.mark_state(target);
        repo::update_if_changed(tx.db(), &before, step, self.ctx.now()).await
    }

    /// A route arriving at a step: finished work there opens again for another pass. A draft is
    /// refused, since a person has not released it.
    pub async fn reopen_for_route_in(&self, tx: &Tx, step_id: Id) -> Result<task_step::Model> {
        let before = repo::task_step::require(tx.db(), step_id).await?;
        if before.state == TaskStepState::Draft {
            return Err(RekallError::illegal(format!(
                "The route leads to '{}', which is still a draft. Promote it in the console before the flow can reach it.",
                before.title
            )));
        }
        if !before.state.complete() {
            return Ok(before);
        }
        let mut step = before.clone();
        step.mark_state(TaskStepState::Open);
        repo::update_if_changed(tx.db(), &before, step, self.ctx.now()).await
    }

    pub async fn title_of(&self, step_id: Option<Id>) -> Result<Option<String>> {
        let Some(step_id) = step_id else {
            return Ok(None);
        };
        in_read!(&self.ctx, |tx| {
            Ok::<_, RekallError>(repo::task_step::find_by_id(tx.db(), step_id).await?.map(|s| s.title))
        })
    }

    /// Move a step `OPEN -> RUNNING` when a session opens on it. A no-op past `OPEN`.
    pub async fn mark_running(&self, step_id: Option<Id>) -> Result<()> {
        self.move_if(step_id, TaskStepState::Open, TaskStepState::Running).await
    }

    /// Drop a step `RUNNING -> OPEN` when its session ends without claiming it. A no-op otherwise.
    pub async fn release_running(&self, step_id: Option<Id>) -> Result<()> {
        self.move_if(step_id, TaskStepState::Running, TaskStepState::Open).await
    }

    async fn move_if(&self, step_id: Option<Id>, from: TaskStepState, to: TaskStepState) -> Result<()> {
        let Some(step_id) = step_id else {
            return Ok(());
        };
        in_write!(&self.ctx, |tx| {
            if let Some(before) = repo::task_step::find_by_id(tx.db(), step_id).await? {
                if before.state == from {
                    let mut step = before.clone();
                    step.mark_state(to);
                    repo::update_if_changed(tx.db(), &before, step, self.ctx.now()).await?;
                    self.publish_in(&mut tx, before.task_id).await?;
                }
            }
            Ok::<_, RekallError>(())
        })
    }

    pub async fn move_to(&self, id: Id, position: i32) -> Result<Vec<TaskStepView>> {
        in_write!(&self.ctx, |tx| {
            let step = self.require(&tx, id).await?;
            let task_id = step.task_id;
            let mut ordered = load::steps_of(tx.db(), task_id).await?;
            ordered.retain(|candidate| candidate.id != id);
            let at = position.clamp(0, ordered.len() as i32) as usize;
            ordered.insert(at, step);
            self.settle_drafts_at_tail(&tx, ordered).await?;
            let settled = self.find_by_task_in(&mut tx, task_id).await?;
            self.publish_in(&mut tx, task_id).await?;
            Ok(settled)
        })
    }

    /// Remembers where the person dragged a step on the flow canvas. `None` hands it back to the
    /// automatic layout. Answers the task's whole checklist, like a route does.
    pub async fn place_in_flow(&self, id: Id, place: Option<FlowPlace>) -> Result<Vec<TaskStepView>> {
        in_write!(&self.ctx, |tx| {
            let before = self.require(&tx, id).await?;
            let mut step = before.clone();
            step.flow_x = place.map(|at| at.x);
            step.flow_y = place.map(|at| at.y);
            repo::update_if_changed(tx.db(), &before, step, self.ctx.now()).await?;
            self.publish_in(&mut tx, before.task_id).await?;
            self.find_by_task_in(&mut tx, before.task_id).await
        })
    }

    /// Caps how many times a flow may enter the step; `None` lifts the cap. Answers the task's
    /// whole checklist, like a route does.
    pub async fn limit_visits(&self, id: Id, budget: Option<VisitBudget>) -> Result<Vec<TaskStepView>> {
        in_write!(&self.ctx, |tx| {
            let saved = self.limit_visits_in(&tx, id, budget).await?;
            self.publish_in(&mut tx, saved.task_id).await?;
            self.find_by_task_in(&mut tx, saved.task_id).await
        })
    }

    pub async fn limit_visits_in(&self, tx: &Tx, id: Id, budget: Option<VisitBudget>) -> Result<task_step::Model> {
        let before = self.require(tx, id).await?;
        let mut step = before.clone();
        step.max_visits = budget.map(VisitBudget::visits);
        repo::update_if_changed(tx.db(), &before, step, self.ctx.now()).await
    }

    /// Sets the connection the step reaches, as a deployment graph laid on the task says.
    pub async fn reach_connection_in(&self, tx: &Tx, id: Id, connection_id: Option<Id>) -> Result<task_step::Model> {
        let before = self.require(tx, id).await?;
        let step = task_step::Model { connection_id, ..before.clone() };
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
            self.publish_in(&mut tx, task_id).await?;
            self.find_by_task_in(&mut tx, task_id).await
        })
    }

    pub async fn delete(&self, id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            if let Some(step) = repo::task_step::find_by_id(tx.db(), id).await? {
                task_step::Entity::delete_by_id(id).exec(tx.db()).await?;
                let remaining = load::steps_of(tx.db(), step.task_id).await?;
                self.renumber(&tx, remaining).await?;
                self.publish_in(&mut tx, step.task_id).await?;
            }
            Ok::<_, RekallError>(())
        })
    }

    /// Keep every draft after every non-draft, the order within each group kept, and renumber.
    async fn settle_drafts_at_tail(&self, tx: &Tx, ordered: Vec<task_step::Model>) -> Result<()> {
        let (drafts, work): (Vec<_>, Vec<_>) = ordered.into_iter().partition(|step| step.state.draft());
        let mut settled = work;
        settled.extend(drafts);
        self.renumber(tx, settled).await?;
        Ok(())
    }

    /// Dense positions from zero. Only a step whose position actually moves is written and stamped.
    async fn renumber(&self, tx: &Tx, ordered: Vec<task_step::Model>) -> Result<Vec<task_step::Model>> {
        let mut out = Vec::with_capacity(ordered.len());
        for (at, before) in ordered.into_iter().enumerate() {
            let mut step = before.clone();
            step.position = at as i32;
            out.push(repo::update_if_changed(tx.db(), &before, step, self.ctx.now()).await?);
        }
        Ok(out)
    }

    /// Announces the task's whole checklist once the transaction commits. Route and visit writes
    /// call it too, since both travel on the step views.
    pub async fn publish_in(&self, tx: &mut Tx, task_id: Id) -> Result<()> {
        let current = self.find_by_task_in(tx, task_id).await?;
        tx.publish(StepStreamEvent { task_id, steps: current });
        Ok(())
    }

    pub async fn view_in(&self, tx: &Tx, step: &task_step::Model) -> Result<TaskStepView> {
        TaskStepView::of(step, &StepFlowRows::of_task(tx.db(), step.task_id).await?)
    }

    async fn require(&self, tx: &Tx, id: Id) -> Result<task_step::Model> {
        repo::task_step::find_by_id(tx.db(), id)
            .await?
            .ok_or_else(|| RekallError::unknown_anchor(format!("No step with id {id}")))
    }
}

/// Move a step between `DRAFT` and `OPEN`. Only those two states move; a step a session has
/// started, claimed or finished is refused.
fn apply_draft(step: &mut task_step::Model, draft: bool) -> Result<()> {
    let current = step.state;
    if draft && current == TaskStepState::Open {
        step.mark_state(TaskStepState::Draft);
    } else if !draft && current == TaskStepState::Draft {
        step.mark_state(TaskStepState::Open);
    } else if draft != (current == TaskStepState::Draft) {
        return Err(RekallError::illegal(format!(
            "'{}' is {} and cannot change its draft state. Only a step that is still draft or open moves between the two.",
            step.title,
            current.name().to_lowercase()
        )));
    }
    Ok(())
}

/// An epic gathers tasks and holds no checklist; its tasks carry the steps.
fn require_not_epic(task: &task::Model) -> Result<()> {
    if task.is_epic() {
        return Err(RekallError::illegal(format!(
            "'{}' is an epic, and an epic holds no steps. Add the step to one of the tasks that belong to it.",
            task.label
        )));
    }
    Ok(())
}

fn validated_title(title: Option<&str>) -> Result<String> {
    let title =
        text_rule::present(title).ok_or_else(|| RekallError::illegal("A step needs a title. To remove one, delete it."))?;
    text_rule::within(title, TITLE_MAX_CHARACTERS, |length| {
        format!(
            "A step's title is capped at {TITLE_MAX_CHARACTERS} characters and this one is {length}. What it has to \
             satisfy goes in the detail below it."
        )
    })
}

/// A step's detail is optional: blank clears it.
fn validated_body(body: Option<&str>) -> Result<Option<String>> {
    text_rule::present(body)
        .map(|body| {
            text_rule::within(body, MAX_CHARACTERS, |length| {
                format!(
                    "A step's detail is capped at {MAX_CHARACTERS} characters and this one is {length}. A step that \
                     long is a task of its own; split it, or move the detail into a note."
                )
            })
        })
        .transpose()
}
