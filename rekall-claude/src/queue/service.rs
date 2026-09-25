//! The whole write path of the run queue: what the console edits (the items, their order, the
//! settings, start and stop) and the moves the runner makes as it works through it. Every write
//! publishes the resulting view as a `run-queue` event once it commits.
//!
//! It lives here, not in `rekall-service`, because it is a console write the MCP module must not
//! reach. The rules it keeps: a task is queued once while it waits or runs; the running item is
//! not removed or moved, it is stopped; a queue with nothing waiting does not start; a start time
//! in the past is refused rather than run at once; the settings apply from the next session opened
//! and the next ceiling check.

use chrono::TimeDelta;
use rekall_common::{jstr, Id, Instant, RekallError, Result};
use rekall_model::{run_queue, run_queue_item, RunQueueItemState, RunQueueState};
use rekall_repository::repository as repo;
use rekall_service::{in_read, in_write, Ctx, DomainEvent, Tx};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};

use super::view::{RunQueueItemView, RunQueueView};
use crate::pty::{EFFORT_LEVELS, MODEL_ALIASES};

pub const CEILING_MIN: i32 = 1;
pub const CEILING_MAX: i32 = 100;

/// How far in the past a start time may be and still count as "now": a form submitted slowly.
const START_SLACK: TimeDelta = TimeDelta::minutes(1);

/// Settings and state as the runner reads them at a decision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub state: RunQueueState,
    pub start_at: Option<Instant>,
    pub hold_until: Option<Instant>,
    pub ceiling_percent: Option<i32>,
    pub skip_permissions: bool,
    pub model: Option<String>,
    pub effort: Option<String>,
}

#[derive(Clone)]
pub struct RunQueueService {
    ctx: Ctx,
}

impl RunQueueService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    fn now(&self) -> Instant {
        self.ctx.now()
    }

    // ---------------------------------------------------------------- console

    pub async fn view(&self) -> Result<RunQueueView> {
        in_write!(&self.ctx, |tx| {
            let queue = self.queue(&tx).await?;
            view_of(&tx, &queue).await
        })
    }

    pub async fn update_settings(
        &self,
        ceiling_percent: Option<i32>,
        skip_permissions: bool,
        model: Option<&str>,
        effort: Option<&str>,
    ) -> Result<RunQueueView> {
        in_write!(&self.ctx, |tx| {
            if ceiling_percent.is_some_and(|c| !(CEILING_MIN..=CEILING_MAX).contains(&c)) {
                return Err(RekallError::illegal(format!(
                    "The ceiling is a usage percentage between {CEILING_MIN} and {CEILING_MAX}, or none."
                )));
            }
            let mut queue = self.queue(&tx).await?;
            queue.ceiling_percent = ceiling_percent;
            queue.skip_permissions = skip_permissions;
            queue.model = choice(model, &MODEL_ALIASES, "model")?;
            queue.effort = choice(effort, &EFFORT_LEVELS, "effort")?;
            self.publish(&mut tx, queue).await
        })
    }

    pub async fn add(&self, task_id: Id) -> Result<RunQueueView> {
        in_write!(&self.ctx, |tx| {
            let task = repo::task::find_by_id(tx.db(), task_id)
                .await?
                .ok_or_else(|| RekallError::unknown_anchor(format!("No task with id {task_id}")))?;
            let current = ordered(&tx).await?;
            let waiting = current.iter().any(|item| item.task_id == task_id && !item.state.settled());
            if waiting {
                return Err(RekallError::conflict(format!("'{}' is already in the queue.", task.title)));
            }
            run_queue_item::Model::new(task_id, current.len() as i32, self.now())
                .into_active_model()
                .insert(tx.db())
                .await?;
            let queue = self.queue(&tx).await?;
            self.publish(&mut tx, queue).await
        })
    }

    pub async fn remove(&self, item_id: Id) -> Result<RunQueueView> {
        in_write!(&self.ctx, |tx| {
            let item = require_item(&tx, item_id).await?;
            if item.state == RunQueueItemState::Running {
                let title = task_title(&tx, item.task_id).await?;
                return Err(RekallError::conflict(format!("'{title}' is running. Stop the queue to take it off.")));
            }
            run_queue_item::Entity::delete_by_id(item.id).exec(tx.db()).await?;
            renumber(&tx, ordered(&tx).await?).await?;
            let queue = self.queue(&tx).await?;
            self.publish(&mut tx, queue).await
        })
    }

    /// Move a waiting item to `index` among the waiting items. Settled and running items keep
    /// their places ahead of them; only the order still to run changes.
    pub async fn move_to(&self, item_id: Id, index: i32) -> Result<RunQueueView> {
        in_write!(&self.ctx, |tx| {
            let item = require_item(&tx, item_id).await?;
            if item.state != RunQueueItemState::Queued {
                return Err(RekallError::conflict("Only a task still waiting its turn can be moved."));
            }
            let all = ordered(&tx).await?;
            let mut waiting: Vec<run_queue_item::Model> = all
                .iter()
                .filter(|candidate| candidate.state == RunQueueItemState::Queued && candidate.id != item.id)
                .cloned()
                .collect();
            let at = index.clamp(0, waiting.len() as i32) as usize;
            waiting.insert(at, item);
            let mut reordered: Vec<run_queue_item::Model> =
                all.iter().filter(|candidate| candidate.state != RunQueueItemState::Queued).cloned().collect();
            reordered.extend(waiting);
            renumber(&tx, reordered).await?;
            let queue = self.queue(&tx).await?;
            self.publish(&mut tx, queue).await
        })
    }

    /// Take every finished, skipped and failed item off the list.
    pub async fn clear_settled(&self) -> Result<RunQueueView> {
        in_write!(&self.ctx, |tx| {
            let all = ordered(&tx).await?;
            let settled: Vec<Id> = all.iter().filter(|item| item.state.settled()).map(|item| item.id).collect();
            if !settled.is_empty() {
                run_queue_item::Entity::delete_many()
                    .filter(run_queue_item::Column::Id.is_in(settled))
                    .exec(tx.db())
                    .await?;
            }
            renumber(&tx, all.into_iter().filter(|item| !item.state.settled()).collect()).await?;
            let queue = self.queue(&tx).await?;
            self.publish(&mut tx, queue).await
        })
    }

    /// Arm the queue: at once when `start_at` is absent or already here, otherwise at `start_at`.
    /// A scheduled queue can be started again to move or drop its time; a running or holding one
    /// cannot.
    pub async fn start(&self, start_at: Option<Instant>) -> Result<RunQueueView> {
        in_write!(&self.ctx, |tx| {
            let mut queue = self.queue(&tx).await?;
            if matches!(queue.state, RunQueueState::Running | RunQueueState::Holding) {
                return Err(RekallError::conflict("The queue is already running. Stop it first to start it again."));
            }
            let any_waiting = ordered(&tx).await?.iter().any(|item| item.state == RunQueueItemState::Queued);
            if !any_waiting {
                return Err(RekallError::conflict("Nothing is waiting in the queue. Add a task first."));
            }
            let now = self.now();
            if start_at.is_some_and(|at| at.is_before(&now.minus(START_SLACK))) {
                return Err(RekallError::illegal("That start time has already passed. Pick a later one, or start now."));
            }
            match start_at {
                Some(at) if at.is_after(&now) => {
                    queue.move_to(RunQueueState::Scheduled);
                    queue.start_at = Some(at);
                }
                _ => queue.move_to(RunQueueState::Running),
            }
            self.publish(&mut tx, queue).await
        })
    }

    /// Disarm the queue. A running item goes back to waiting, at the head, with the reason.
    pub async fn stop(&self, reason: &str) -> Result<RunQueueView> {
        in_write!(&self.ctx, |tx| {
            let mut queue = self.queue(&tx).await?;
            queue.move_to(RunQueueState::Idle);
            for mut item in ordered(&tx).await?.into_iter().filter(|item| item.state == RunQueueItemState::Running) {
                item.move_to(RunQueueItemState::Queued, Some(reason));
                save_item(&tx, item).await?;
            }
            self.publish(&mut tx, queue).await
        })
    }

    // ---------------------------------------------------------------- runner

    pub async fn snapshot(&self) -> Result<Snapshot> {
        in_read!(&self.ctx, |tx| {
            Ok::<_, RekallError>(match repo::run_queue::find_first_by_order_by_created_at_asc(tx.db()).await? {
                None => Snapshot {
                    state: RunQueueState::Idle,
                    start_at: None,
                    hold_until: None,
                    ceiling_percent: None,
                    skip_permissions: false,
                    model: None,
                    effort: None,
                },
                Some(queue) => Snapshot {
                    state: queue.state,
                    start_at: queue.start_at,
                    hold_until: queue.hold_until,
                    ceiling_percent: queue.ceiling_percent,
                    skip_permissions: queue.skip_permissions,
                    model: queue.model,
                    effort: queue.effort,
                },
            })
        })
    }

    /// The first item still waiting, if any: the next one to run.
    pub async fn next_waiting(&self) -> Result<Option<RunQueueItemView>> {
        in_read!(&self.ctx, |tx| {
            match ordered(&tx).await?.into_iter().find(|item| item.state == RunQueueItemState::Queued) {
                None => Ok(None),
                Some(item) => Ok(Some(item_view(&tx, &item).await?)),
            }
        })
    }

    /// `SCHEDULED` or `HOLDING` to `RUNNING`: the time came, or the window reset.
    pub async fn resume(&self) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let mut queue = self.queue(&tx).await?;
            if matches!(queue.state, RunQueueState::Scheduled | RunQueueState::Holding) {
                queue.move_to(RunQueueState::Running);
                self.publish(&mut tx, queue).await?;
            }
            Ok(())
        })
    }

    pub async fn hold(&self, until: Option<Instant>, reason: Option<&str>) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let mut queue = self.queue(&tx).await?;
            if !queue.state.armed() {
                return Ok(());
            }
            queue.move_to(RunQueueState::Holding);
            queue.hold_until = until;
            queue.hold_reason = reason.map(str::to_string);
            self.publish(&mut tx, queue).await?;
            Ok(())
        })
    }

    /// The last waiting item has had its turn: back to `IDLE`.
    pub async fn finish(&self) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let mut queue = self.queue(&tx).await?;
            if queue.state.armed() {
                queue.move_to(RunQueueState::Idle);
                self.publish(&mut tx, queue).await?;
            }
            Ok(())
        })
    }

    /// Record an item's move. An item deleted in the meantime (its task was) is ignored.
    pub async fn mark_item(&self, item_id: Id, state: RunQueueItemState, reason: Option<&str>) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            if let Some(mut item) = repo_item(&tx, item_id).await? {
                item.move_to(state, reason);
                save_item(&tx, item).await?;
                let queue = self.queue(&tx).await?;
                self.publish(&mut tx, queue).await?;
            }
            Ok(())
        })
    }

    /// After a restart no session survives, so an item recorded as running is waiting again. The
    /// queue keeps its state: an armed queue picks the item back up on the runner's first tick.
    pub async fn recover_after_restart(&self) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let orphaned: Vec<run_queue_item::Model> =
                ordered(&tx).await?.into_iter().filter(|item| item.state == RunQueueItemState::Running).collect();
            let any = !orphaned.is_empty();
            for mut item in orphaned {
                item.move_to(
                    RunQueueItemState::Queued,
                    Some("Rekall restarted while this ran; it picks up at the next open step."),
                );
                save_item(&tx, item).await?;
            }
            if any {
                let queue = self.queue(&tx).await?;
                self.publish(&mut tx, queue).await?;
            }
            Ok(())
        })
    }

    // ---------------------------------------------------------------- helpers

    /// The one queue row, created the first time it is asked for.
    async fn queue(&self, tx: &Tx) -> Result<run_queue::Model> {
        if let Some(queue) = repo::run_queue::find_first_by_order_by_created_at_asc(tx.db()).await? {
            return Ok(queue);
        }
        let queue = run_queue::Model::new(self.now());
        queue.clone().into_active_model().insert(tx.db()).await?;
        Ok(queue)
    }

    /// Stamp, save and announce. The stamp moves on every write, an item's included, because the
    /// console orders what it receives by it and drops anything older than what it holds.
    async fn publish(&self, tx: &mut Tx, mut queue: run_queue::Model) -> Result<RunQueueView> {
        queue.updated_at = self.now();
        queue.validate()?;
        queue.clone().into_active_model().reset_all().update(tx.db()).await?;
        let view = view_of(tx, &queue).await?;
        tx.publish(DomainEvent::Broadcast {
            name: "run-queue".into(),
            payload: serde_json::to_value(&view).unwrap_or_default(),
        });
        Ok(view)
    }
}

async fn ordered(tx: &Tx) -> Result<Vec<run_queue_item::Model>> {
    repo::run_queue::find_all_items_by_order_by_position_asc(tx.db()).await
}

async fn repo_item(tx: &Tx, id: Id) -> Result<Option<run_queue_item::Model>> {
    Ok(run_queue_item::Entity::find_by_id(id).one(tx.db()).await?)
}

async fn require_item(tx: &Tx, id: Id) -> Result<run_queue_item::Model> {
    repo_item(tx, id)
        .await?
        .ok_or_else(|| RekallError::not_found_msg(format!("No queued task with id {id}")))
}

async fn save_item(tx: &Tx, item: run_queue_item::Model) -> Result<()> {
    item.into_active_model().reset_all().update(tx.db()).await?;
    Ok(())
}

async fn renumber(tx: &Tx, ordered: Vec<run_queue_item::Model>) -> Result<()> {
    for (at, mut item) in ordered.into_iter().enumerate() {
        if item.position != at as i32 {
            item.position = at as i32;
            save_item(tx, item).await?;
        }
    }
    Ok(())
}

async fn task_title(tx: &Tx, task_id: Id) -> Result<String> {
    Ok(repo::task::find_by_id(tx.db(), task_id).await?.map(|t| t.title).unwrap_or_default())
}

async fn item_view(tx: &Tx, item: &run_queue_item::Model) -> Result<RunQueueItemView> {
    let task = repo::task::find_by_id(tx.db(), item.task_id)
        .await?
        .ok_or_else(|| RekallError::unknown_anchor(format!("No task with id {}", item.task_id)))?;
    let project = repo::project::find_by_id(tx.db(), task.project_id)
        .await?
        .ok_or_else(|| RekallError::internal("IllegalStateException", "A task has no project"))?;
    Ok(RunQueueItemView::of(item, &task, &project))
}

async fn view_of(tx: &Tx, queue: &run_queue::Model) -> Result<RunQueueView> {
    let mut items = Vec::new();
    for item in ordered(tx).await? {
        items.push(item_view(tx, &item).await?);
    }
    Ok(RunQueueView::of(queue, items))
}

/// Blank or `default` is the account's own setting; anything else must be offered.
fn choice(value: Option<&str>, allowed: &[&str], what: &str) -> Result<Option<String>> {
    let Some(value) = value else { return Ok(None) };
    if jstr::is_blank(value) || jstr::equals_ignore_case(jstr::strip(value), "default") {
        return Ok(None);
    }
    let normalised = jstr::strip(value).to_lowercase();
    if !allowed.contains(&normalised.as_str()) {
        return Err(RekallError::illegal(format!("'{value}' is not a {what} Claude Code accepts.")));
    }
    Ok(Some(normalised))
}
