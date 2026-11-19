use rekall_kernel::{Id, RekallError, Result};
use rekall_store::task;
use rekall_store::TaskStepState;
use rekall_store::repository as repo;

use crate::step::TaskStepService;
use crate::timeentry::TimeEntryService;
use crate::{in_write, load, Ctx, Tx};

use crate::review::{TaskReviewEvent, TaskReviewView};

#[derive(Clone)]
pub struct TaskReviewService {
    ctx: Ctx,
    time_entries: TimeEntryService,
    steps: TaskStepService,
}

impl TaskReviewService {
    pub fn new(ctx: Ctx, time_entries: TimeEntryService, steps: TaskStepService) -> Self {
        Self { ctx, time_entries, steps }
    }

    /// Flip `OPEN <-> RUNNING` as a session on the task anchor comes and goes, and say whether it
    /// flipped. Ambient: never blocks a claim.
    pub async fn session_running(&self, task_id: Id, running: bool) -> Result<bool> {
        in_write!(&self.ctx, |tx| {
            let Some(before) = repo::task::find_by_id(tx.db(), task_id).await? else {
                return Ok(false);
            };
            if !load::review_active(tx.db(), task_id).await? {
                return Ok(false);
            }
            let mut task = before.clone();
            if running && task.review_state == TaskStepState::Open {
                task.mark_review_state(TaskStepState::Running);
                self.save_and_publish(&mut tx, &before, task).await?;
                return Ok(true);
            }
            if !running && task.review_state == TaskStepState::Running {
                task.mark_review_state(TaskStepState::Open);
                self.save_and_publish(&mut tx, &before, task).await?;
                return Ok(true);
            }
            Ok(false)
        })
    }

    /// A Claude-authored wrapup advances `OPEN | RUNNING -> CLAIMED`. A hand-written one does not.
    pub async fn claimed_by_wrapup_in(&self, tx: &mut Tx, task_id: Id) -> Result<()> {
        let Some(before) = repo::task::find_by_id(tx.db(), task_id).await? else {
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

    /// The console accepts the work: `-> DONE` from any state but `DONE` itself. Accepting a claim
    /// also stops the task's timer: the work it measured is over.
    pub async fn accept_in(&self, tx: &mut Tx, task_id: Id) -> Result<TaskReviewView> {
        let before = load::task_or_unknown(tx.db(), task_id).await?;
        guard_active(tx, task_id).await?;
        let mut task = before.clone();
        task.mark_review_state(TaskStepState::Done);
        self.settle_acceptance(tx, before, task).await
    }

    /// The console accepts the task: its review line when it has no checklist, else every claimed step at once.
    pub async fn accept_work_in(&self, tx: &mut Tx, task_id: Id) -> Result<()> {
        if load::review_active(tx.db(), task_id).await? {
            return self.accept_in(tx, task_id).await.map(|_| ());
        }
        self.steps.accept_claimed_in(tx, task_id).await
    }

    /// The task is put to done: its review line when it has no checklist, else every step that is
    /// not a draft. An epic holds neither, and what is already accepted stays as it is.
    pub async fn accept_everything_in(&self, tx: &mut Tx, task_id: Id) -> Result<()> {
        let task = load::task_or_unknown(tx.db(), task_id).await?;
        if task.is_epic() {
            return Ok(());
        }
        if !load::review_active(tx.db(), task_id).await? {
            return self.steps.accept_all_in(tx, task_id).await;
        }
        if task.review_state != TaskStepState::Done {
            self.accept_in(tx, task_id).await?;
        }
        Ok(())
    }

    /// A quick task that a Claude-authored wrapup has claimed is accepted without a review, marked
    /// `auto_accepted` so the console can reopen it. Says whether it was.
    pub async fn accept_quick_claim(&self, task_id: Id) -> Result<bool> {
        in_write!(&self.ctx, |tx| {
            let Some(before) = repo::task::find_by_id(tx.db(), task_id).await? else {
                return Ok(false);
            };
            let claimed_quick = before.quick && before.review_state == TaskStepState::Claimed;
            if !claimed_quick || !load::review_active(tx.db(), task_id).await? {
                return Ok(false);
            }
            let mut task = before.clone();
            task.accept_automatically();
            self.settle_acceptance(&mut tx, before, task).await?;
            Ok(true)
        })
    }

    /// The console sends the work back: `-> OPEN`.
    pub async fn send_back_in(&self, tx: &mut Tx, task_id: Id) -> Result<TaskReviewView> {
        let before = load::task_or_unknown(tx.db(), task_id).await?;
        guard_active(tx, task_id).await?;
        let mut task = before.clone();
        task.mark_review_state(TaskStepState::Open);
        self.save_and_publish(tx, &before, task).await
    }

    async fn settle_acceptance(&self, tx: &mut Tx, before: task::Model, accepted: task::Model) -> Result<TaskReviewView> {
        if before.review_state == TaskStepState::Done {
            return Err(RekallError::illegal(
                "This task is already accepted. Send it back to reopen it for another pass.",
            ));
        }
        let was_claimed = before.review_state == TaskStepState::Claimed;
        let task_id = before.id;
        let view = self.save_and_publish(tx, &before, accepted).await?;
        if was_claimed {
            self.time_entries.stop_if_running_in(tx, task_id).await?;
        }
        Ok(view)
    }

    async fn save_and_publish(&self, tx: &mut Tx, before: &task::Model, task: task::Model) -> Result<TaskReviewView> {
        let task = repo::update_if_changed(tx.db(), before, task, self.ctx.now()).await?;
        let view = TaskReviewView::of(&task, load::review_active(tx.db(), task.id).await?);
        tx.publish(TaskReviewEvent { task_id: task.id, review: view.clone() });
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
