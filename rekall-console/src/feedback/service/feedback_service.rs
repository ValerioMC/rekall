use rekall_core::feedback::FeedbackStreamEvent;
use rekall_core::review::TaskReviewService;
use rekall_core::{in_read, in_write, Ctx, Tx};
use rekall_kernel::{Id, RekallError, Result};
use rekall_store::repository as repo;
use rekall_store::{task, task_feedback, FeedbackKind, TaskStepState};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use crate::feedback::rule::revision_rule::{decide, is_evaluated, OpenRevision, RevisionDecision};
use crate::feedback::{AddFeedbackCommand, FeedbackView};

#[derive(Clone)]
pub struct FeedbackService {
    ctx: Ctx,
    review: TaskReviewService,
}

impl FeedbackService {
    pub fn new(ctx: Ctx, review: TaskReviewService) -> Self {
        Self { ctx, review }
    }

    pub async fn list(&self, task_id: Id) -> Result<Vec<FeedbackView>> {
        in_read!(&self.ctx, |tx| {
            repo::task::require(tx.db(), task_id).await?;
            let thread = repo::task_feedback::list_of_task(tx.db(), task_id).await?;
            Ok(thread.iter().map(FeedbackView::of).collect())
        })
    }

    /// Feedback on the work of a task with no steps whose review line is claimed or accepted
    /// reopens that line, as Send back did.
    pub async fn add(&self, task_id: Id, command: AddFeedbackCommand) -> Result<FeedbackView> {
        in_write!(&self.ctx, |tx| {
            require_typed_kind(command.kind)?;
            let task = repo::task::require(tx.db(), task_id).await?;
            let seq = repo::task_feedback::next_seq(tx.db(), task_id).await?;
            let entry = task_feedback::Model::new(task_id, seq, command.kind, None, command.body.trim().to_string(), self.ctx.now());
            entry.validate()?;
            entry.clone().into_active_model().insert(tx.db()).await?;
            if command.kind == FeedbackKind::Implementation {
                self.reopen_review_line_in(&mut tx, &task).await?;
            }
            tx.publish(FeedbackStreamEvent { task_id });
            Ok(FeedbackView::of(&entry))
        })
    }

    pub async fn edit(&self, feedback_id: Id, body: &str) -> Result<FeedbackView> {
        in_write!(&self.ctx, |tx| {
            let before = require_pending_entry(tx.db(), feedback_id).await?;
            require_not_a_revision(&before, "edited")?;
            let mut after = before.clone();
            after.body_markdown = body.trim().to_string();
            let saved = repo::update_if_changed(tx.db(), &before, after, self.ctx.now()).await?;
            tx.publish(FeedbackStreamEvent { task_id: saved.task_id });
            Ok(FeedbackView::of(&saved))
        })
    }

    pub async fn delete(&self, feedback_id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let entry = require_pending_entry(tx.db(), feedback_id).await?;
            require_not_a_revision(&entry, "deleted")?;
            task_feedback::Entity::delete_by_id(entry.id).exec(tx.db()).await?;
            tx.publish(FeedbackStreamEvent { task_id: entry.task_id });
            Ok::<_, RekallError>(())
        })
    }

    /// Puts the description as it was before the revision back, and removes the revision.
    pub async fn discard_revision(&self, feedback_id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let entry = require_pending_entry(tx.db(), feedback_id).await?;
            let Some(quoted) = entry.quoted_markdown.clone().filter(|_| entry.kind.is_revision()) else {
                return Err(RekallError::conflict("Only a revision can be discarded; feedback on the work or the plan is deleted."));
            };
            let before = repo::task::require(tx.db(), entry.task_id).await?;
            let restored = task::Model { description: Some(quoted).filter(|text| !text.trim().is_empty()), ..before.clone() };
            repo::update_if_changed(tx.db(), &before, restored, self.ctx.now()).await?;
            task_feedback::Entity::delete_by_id(entry.id).exec(tx.db()).await?;
            tx.publish(FeedbackStreamEvent { task_id: entry.task_id });
            Ok::<_, RekallError>(())
        })
    }

    /// Called by the task service inside the transaction that saves a changed description.
    pub async fn track_description_in(&self, tx: &mut Tx, task: &task::Model, before: Option<&str>, after: Option<&str>) -> Result<()> {
        let evaluated = self.is_task_evaluated(tx, task).await?;
        let open = repo::task_feedback::latest_open_revision(tx.db(), task.id).await?;
        let open_revision = open.as_ref().and_then(|entry| entry.quoted_markdown.as_deref()).map(|quoted_markdown| OpenRevision { quoted_markdown });
        match decide(evaluated, open_revision, before, after) {
            RevisionDecision::Ignore => return Ok(()),
            RevisionDecision::Open { quoted, body } => {
                let seq = repo::task_feedback::next_seq(tx.db(), task.id).await?;
                let revision = task_feedback::Model::new(task.id, seq, FeedbackKind::Revision, Some(quoted), body, self.ctx.now());
                revision.validate()?;
                revision.into_active_model().insert(tx.db()).await?;
            }
            RevisionDecision::Update { body } => {
                let before = open.expect("an update is decided only for an open revision");
                let after = task_feedback::Model { body_markdown: body, ..before.clone() };
                repo::update_if_changed(tx.db(), &before, after, self.ctx.now()).await?;
            }
            RevisionDecision::Remove => {
                let revision = open.expect("a removal is decided only for an open revision");
                task_feedback::Entity::delete_by_id(revision.id).exec(tx.db()).await?;
            }
        }
        tx.publish(FeedbackStreamEvent { task_id: task.id });
        Ok(())
    }

    async fn is_task_evaluated(&self, tx: &Tx, task: &task::Model) -> Result<bool> {
        let has_wrapup = repo::wrapup::find_of_task(tx.db(), task.id).await?.is_some();
        let steps = repo::task_step::list_of_task(tx.db(), task.id).await?;
        Ok(is_evaluated(has_wrapup, steps.iter().map(|step| &step.state), task.review_state))
    }

    async fn reopen_review_line_in(&self, tx: &mut Tx, task: &task::Model) -> Result<()> {
        let steps = repo::task_step::list_of_task(tx.db(), task.id).await?;
        let line_is_closed = matches!(task.review_state, TaskStepState::Claimed | TaskStepState::Done);
        if line_is_closed && task::review_active(steps.iter().map(|step| &step.state)) {
            self.review.send_back_in(tx, task.id, None).await?;
        }
        Ok(())
    }
}

fn require_typed_kind(kind: FeedbackKind) -> Result<()> {
    if kind.is_revision() {
        return Err(RekallError::illegal("A revision is never written: it comes from saving the description. Feedback is on the work (IMPLEMENTATION) or on the plan (PLAN)."));
    }
    Ok(())
}

async fn require_pending_entry(db: &impl sea_orm::ConnectionTrait, feedback_id: Id) -> Result<task_feedback::Model> {
    let entry = repo::task_feedback::find_by_id(db, feedback_id).await?.ok_or_else(|| RekallError::not_found("Feedback", feedback_id))?;
    if !entry.pending() {
        return Err(RekallError::conflict("This entry was already applied by a session and can no longer be changed."));
    }
    Ok(entry)
}

fn require_not_a_revision(entry: &task_feedback::Model, action: &str) -> Result<()> {
    if entry.kind.is_revision() {
        return Err(RekallError::conflict(format!("A revision follows the description and cannot be {action}; discard it to put the earlier text back.")));
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/unit/feedback/service/feedback_service_tests.rs"]
mod tests;
