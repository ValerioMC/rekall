//! The task-scoped mirror of the step line: walks a stepless task along
//! `OPEN -> RUNNING -> CLAIMED -> DONE` and pushes every move onto the step feed as a
//! `task-review` event. `session_running` and `claimed_by_wrapup` ride on existing signals;
//! `accept` and `send_back` are the console's alone, with no MCP tool. Every method is inert once
//! the task has a checklist.

use rekall_common::{Id, Instant, RekallError, Result};
use rekall_model::constraints::Phase;
use rekall_model::task;
use rekall_model::TaskStepState;
use sea_orm::{ActiveModelTrait, IntoActiveModel};
use serde::Serialize;

use crate::{in_write, load, Ctx, DomainEvent, Tx};

/// The review line as the console reads it. `review_active` is false once the task has a
/// checklist, and the rest then means nothing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskReviewView {
    pub task_id: Id,
    pub review_state: TaskStepState,
    pub review_active: bool,
    pub claimed_at: Option<Instant>,
    pub accepted_at: Option<Instant>,
    pub review_note: Option<String>,
}

impl TaskReviewView {
    pub fn of(task: &task::Model, review_active: bool) -> Self {
        Self {
            task_id: task.id,
            review_state: task.review_state,
            review_active,
            claimed_at: task.claimed_at,
            accepted_at: task.accepted_at,
            review_note: task.review_note.clone(),
        }
    }
}

/// Emitted whenever a stepless task's review line moves, over the same feed the steps use.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskReviewEvent {
    pub task_id: Id,
    pub review: TaskReviewView,
}

#[derive(Clone)]
pub struct TaskReviewService {
    ctx: Ctx,
}

impl TaskReviewService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    /// Flip `OPEN <-> RUNNING` as a session on the task anchor comes and goes. Ambient: never
    /// blocks a claim.
    pub async fn session_running(&self, task_id: Id, running: bool) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let Some(before) = rekall_repository::repository::task::find_by_id(tx.db(), task_id).await? else {
                return Ok(());
            };
            if !load::review_active(tx.db(), task_id).await? {
                return Ok(());
            }
            let mut task = before.clone();
            if running && task.review_state == TaskStepState::Open {
                task.mark_review_state(TaskStepState::Running);
                self.save_and_publish(&mut tx, &before, task).await?;
            } else if !running && task.review_state == TaskStepState::Running {
                task.mark_review_state(TaskStepState::Open);
                self.save_and_publish(&mut tx, &before, task).await?;
            }
            Ok(())
        })
    }

    /// A Claude-authored wrapup advances `OPEN | RUNNING -> CLAIMED`. A hand-written one does not.
    pub async fn claimed_by_wrapup_in(&self, tx: &mut Tx, task_id: Id) -> Result<()> {
        let Some(before) = rekall_repository::repository::task::find_by_id(tx.db(), task_id).await? else {
            return Ok(());
        };
        if !load::review_active(tx.db(), task_id).await? {
            return Ok(());
        }
        if matches!(before.review_state, TaskStepState::Open | TaskStepState::Running) {
            let mut task = before.clone();
            task.mark_review_state(TaskStepState::Claimed);
            self.save_and_publish(tx, &before, task).await?;
        }
        Ok(())
    }

    /// The console accepts the work: `-> DONE` from any state but `DONE` itself.
    pub async fn accept_in(&self, tx: &mut Tx, task_id: Id) -> Result<TaskReviewView> {
        let before = load::task_or_unknown(tx.db(), task_id).await?;
        guard_active(tx, task_id).await?;
        if before.review_state == TaskStepState::Done {
            return Err(RekallError::illegal(
                "This task is already accepted. Send it back to reopen it for another pass.",
            ));
        }
        let mut task = before.clone();
        task.mark_review_state(TaskStepState::Done);
        self.save_and_publish(tx, &before, task).await
    }

    /// The console sends the work back: `-> OPEN`, with an optional note; blank notes are dropped.
    pub async fn send_back_in(&self, tx: &mut Tx, task_id: Id, note: Option<&str>) -> Result<TaskReviewView> {
        let before = load::task_or_unknown(tx.db(), task_id).await?;
        guard_active(tx, task_id).await?;
        let mut task = before.clone();
        task.mark_review_state(TaskStepState::Open);
        task.review_note = note
            .filter(|n| !rekall_common::jstr::is_blank(n))
            .map(|n| rekall_common::jstr::strip(n).to_string());
        self.save_and_publish(tx, &before, task).await
    }

    async fn save_and_publish(&self, tx: &mut Tx, before: &task::Model, mut task: task::Model) -> Result<TaskReviewView> {
        if &task != before {
            task.updated_at = self.ctx.now();
            task.validate(Phase::Update)?;
            task.clone().into_active_model().reset_all().update(tx.db()).await?;
        }
        let view = TaskReviewView::of(&task, load::review_active(tx.db(), task.id).await?);
        tx.publish(DomainEvent::TaskReview(TaskReviewEvent { task_id: task.id, review: view.clone() }));
        Ok(view)
    }
}

async fn guard_active(tx: &Tx, task_id: Id) -> Result<()> {
    if !load::review_active(tx.db(), task_id).await? {
        return Err(RekallError::illegal(
            "This task has a checklist, so its steps carry the review, not the task.",
        ));
    }
    Ok(())
}
