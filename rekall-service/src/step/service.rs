use regex::Regex;
use rekall_common::{jstr, Id, RekallError, Result};
use rekall_model::constraints::Phase;
use rekall_model::task_step::{self, MAX_CHARACTERS};
use rekall_model::TaskStepState;
use rekall_repository::repository as repo;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};
use std::sync::LazyLock;

use super::{StepStreamEvent, TaskStepView};
use crate::{in_read, in_write, load, Ctx, DomainEvent, Tx};

/// Drafts a session may leave on one task before a person has promoted or deleted some.
pub const PROPOSED_DRAFTS_MAX: usize = 20;

/// A draft a session proposed, and how many drafts the task now holds.
#[derive(Clone, Debug)]
pub struct Proposed {
    pub step: TaskStepView,
    pub draft_count: usize,
}

#[derive(Clone)]
pub struct TaskStepService {
    ctx: Ctx,
}

impl TaskStepService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn find_all(&self) -> Result<Vec<TaskStepView>> {
        in_read!(&self.ctx, |tx| {
            let steps = repo::task_step::find_all_by_order_by_task_id_asc_position_asc(tx.db()).await?;
            Ok::<_, RekallError>(steps.iter().map(TaskStepView::of).collect())
        })
    }

    pub async fn find_by_task(&self, task_id: Id) -> Result<Vec<TaskStepView>> {
        in_read!(&self.ctx, |tx| self.find_by_task_in(&mut tx, task_id).await)
    }

    pub async fn find_by_task_in(&self, tx: &mut Tx, task_id: Id) -> Result<Vec<TaskStepView>> {
        let steps = load::steps_of(tx.db(), task_id).await?;
        Ok(steps.iter().map(TaskStepView::of).collect())
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
        load::task_or_unknown(tx.db(), task_id).await?;
        let siblings = load::steps_of(tx.db(), task_id).await?;
        let mut step = task_step::Model::new(task_id, validated_title(title)?, siblings.len() as i32, self.ctx.now());
        step.body_markdown = validated_body(body_markdown)?;
        step.validate(Phase::Persist)?;
        let saved = step.into_active_model().insert(tx.db()).await?;
        let view = TaskStepView::of(&saved);
        self.publish(tx, task_id).await?;
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
            self.save(&tx, &before, step).await?;
            if draft.is_some() {
                let ordered = load::steps_of(tx.db(), task_id).await?;
                self.settle_drafts_at_tail(&tx, ordered).await?;
            }
            let saved = TaskStepView::of(&self.require(&tx, id).await?);
            self.publish(&mut tx, task_id).await?;
            Ok(saved)
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
            if target == TaskStepState::Done {
                return Err(RekallError::illegal(
                    "A step is marked done in the console, by the person who reviewed the work. \
                     A session can take it as far as `claimed`.",
                ));
            }
            let task = load::resolve_task(tx.db(), project_label, task_label).await?;
            let before = resolve_step(tx.db(), task.id, step_ref).await?;
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
            let saved = self.save(&tx, &before, step).await?;
            let view = TaskStepView::of(&saved);
            self.publish(&mut tx, task.id).await?;
            Ok(view)
        })
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
                    self.save(&tx, &before, step).await?;
                    self.publish(&mut tx, before.task_id).await?;
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
            let settled = self.settle_drafts_at_tail(&tx, ordered).await?;
            self.publish(&mut tx, task_id).await?;
            Ok(settled)
        })
    }

    pub async fn delete(&self, id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            if let Some(step) = repo::task_step::find_by_id(tx.db(), id).await? {
                task_step::Entity::delete_by_id(id).exec(tx.db()).await?;
                let remaining = load::steps_of(tx.db(), step.task_id).await?;
                self.renumber(&tx, remaining).await?;
                self.publish(&mut tx, step.task_id).await?;
            }
            Ok::<_, RekallError>(())
        })
    }

    /// Keep every draft after every non-draft, the order within each group kept, and renumber.
    async fn settle_drafts_at_tail(&self, tx: &Tx, ordered: Vec<task_step::Model>) -> Result<Vec<TaskStepView>> {
        let (drafts, work): (Vec<_>, Vec<_>) = ordered.into_iter().partition(|step| step.state.draft());
        let mut settled = work;
        settled.extend(drafts);
        let renumbered = self.renumber(tx, settled).await?;
        Ok(renumbered.iter().map(TaskStepView::of).collect())
    }

    /// Dense positions from zero. Only a step whose position actually moves is written, and only
    /// that one has its update stamp moved, as Hibernate's dirty check did.
    async fn renumber(&self, tx: &Tx, ordered: Vec<task_step::Model>) -> Result<Vec<task_step::Model>> {
        let mut out = Vec::with_capacity(ordered.len());
        for (at, before) in ordered.into_iter().enumerate() {
            let mut step = before.clone();
            step.position = at as i32;
            out.push(self.save(tx, &before, step).await?);
        }
        Ok(out)
    }

    /// Write `after` over `before` if anything changed, stamping `updated_at`.
    async fn save(&self, tx: &Tx, before: &task_step::Model, mut after: task_step::Model) -> Result<task_step::Model> {
        if &after == before {
            return Ok(after);
        }
        after.updated_at = self.ctx.now();
        after.validate(Phase::Update)?;
        after.clone().into_active_model().reset_all().update(tx.db()).await?;
        Ok(after)
    }

    async fn publish(&self, tx: &mut Tx, task_id: Id) -> Result<()> {
        let current = self.find_by_task_in(tx, task_id).await?;
        tx.publish(DomainEvent::Steps(StepStreamEvent { task_id, steps: current }));
        Ok(())
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

/// A step by its one-based number in the checklist, or by its exact title.
async fn resolve_step(db: &impl sea_orm::ConnectionTrait, task_id: Id, reference: Option<&str>) -> Result<task_step::Model> {
    static DIGITS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d+$").unwrap());
    let ordered = load::steps_of(db, task_id).await?;
    if ordered.is_empty() {
        return Err(RekallError::illegal("That task has no steps to move."));
    }
    let trimmed = reference.map(jstr::strip).unwrap_or("");
    if DIGITS.is_match(trimmed) {
        let count = ordered.len();
        // Integer.parseInt: a number past int range is a NumberFormatException, an IAE.
        let one_based: i64 = trimmed
            .parse::<i32>()
            .map_err(|_| RekallError::illegal(format!("For input string: \"{trimmed}\"")))?
            .into();
        if one_based < 1 || one_based > count as i64 {
            return Err(RekallError::illegal(format!("There is no step {one_based}. The checklist has {count}.")));
        }
        return Ok(ordered[(one_based - 1) as usize].clone());
    }
    let by_title: Vec<&task_step::Model> = ordered
        .iter()
        .filter(|step| jstr::equals_ignore_case(jstr::strip(&step.title), trimmed))
        .collect();
    if by_title.len() == 1 {
        return Ok(by_title[0].clone());
    }
    let titles: Vec<String> = ordered
        .iter()
        .enumerate()
        .map(|(at, step)| format!("{}. {}", at + 1, step.title))
        .collect();
    Err(RekallError::illegal(format!(
        "No step matches '{}'. Pass its number or its exact title:\n{}",
        reference.unwrap_or("null"),
        titles.join("\n")
    )))
}

fn validated_title(title: Option<&str>) -> Result<String> {
    let Some(title) = title.filter(|t| !jstr::is_blank(t)) else {
        return Err(RekallError::illegal("A step needs a title. To remove one, delete it."));
    };
    let text = jstr::strip(title);
    let length = jstr::len(text);
    if length > 200 {
        return Err(RekallError::illegal(format!(
            "A step's title is capped at 200 characters and this one is {length}. What it has to satisfy goes in \
             the detail below it."
        )));
    }
    Ok(text.to_string())
}

fn validated_body(body: Option<&str>) -> Result<Option<String>> {
    let Some(body) = body.filter(|b| !jstr::is_blank(b)) else {
        return Ok(None);
    };
    let text = jstr::strip(body);
    let length = jstr::len(text);
    if length > MAX_CHARACTERS {
        return Err(RekallError::illegal(format!(
            "A step's detail is capped at {MAX_CHARACTERS} characters and this one is {length}. A step that long is a \
             task of its own; split it, or move the detail into a note."
        )));
    }
    Ok(Some(text.to_string()))
}
