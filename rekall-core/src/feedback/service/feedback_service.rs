use rekall_kernel::{Id, RekallError, Result};
use rekall_store::repository as repo;
use rekall_store::task_feedback;

use crate::feedback::{FeedbackStreamEvent, Replied};
use crate::{in_write, load, text_rule, Ctx};

const REPLY_MAX_CHARACTERS: usize = task_feedback::REPLY_MAX_CHARACTERS;

/// Answers the open entries of one task's thread, and gives back the ones a session left unanswered
/// when it ends. An applied revision changes nothing else: the description already holds its text.
#[derive(Clone)]
pub struct FeedbackService {
    ctx: Ctx,
}

impl FeedbackService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn reply(&self, project_label: Option<&str>, task_label: &str, replies: Vec<(i32, String)>) -> Result<Replied> {
        in_write!(&self.ctx, |tx| {
            let task = load::resolve_task(tx.db(), project_label, task_label).await?;
            if replies.is_empty() {
                return Err(RekallError::illegal("A reply names at least one pending entry by its seq. Nothing was written."));
            }
            let now = self.ctx.now();
            let mut applied = Vec::new();
            for (seq, text) in &replies {
                let entry = repo::task_feedback::find_by_seq(tx.db(), task.id, *seq).await?.ok_or_else(|| unknown_entry(*seq))?;
                if !entry.open() {
                    return Err(already_applied(*seq));
                }
                let reply = validated_reply(*seq, text)?;
                let before = entry.clone();
                let mut after = entry;
                after.mark_applied(reply, now);
                repo::update_if_changed(tx.db(), &before, after, now).await?;
                applied.push(*seq);
            }
            let still_open = repo::task_feedback::open_of_task(tx.db(), task.id).await?.iter().map(|entry| entry.seq).collect();
            tx.publish(FeedbackStreamEvent { task_id: task.id });
            Ok(Replied { task_anchor: task.anchor(), applied, still_open })
        })
    }

    /// The session that took a pass ended: entries it never answered wait for the next run again.
    pub async fn release_running(&self, task_id: Id) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let running = repo::task_feedback::running_of_task(tx.db(), task_id).await?;
            if running.is_empty() {
                return Ok(());
            }
            let now = self.ctx.now();
            for before in running {
                let mut after = before.clone();
                after.release(now);
                repo::update_if_changed(tx.db(), &before, after, now).await?;
            }
            tx.publish(FeedbackStreamEvent { task_id });
            Ok(())
        })
    }
}

fn unknown_entry(seq: i32) -> RekallError {
    RekallError::illegal(format!("This task's thread has no entry with seq {seq}. Nothing was written."))
}

fn already_applied(seq: i32) -> RekallError {
    RekallError::illegal(format!("Entry {seq} is already applied and cannot be replied to again. Nothing was written."))
}

fn validated_reply(seq: i32, text: &str) -> Result<String> {
    let reply = text_rule::present(Some(text)).ok_or_else(|| RekallError::illegal(format!("The reply to entry {seq} is blank: say what changed and where.")))?;
    text_rule::within(reply, REPLY_MAX_CHARACTERS, |length| {
        format!("The reply to entry {seq} is capped at {REPLY_MAX_CHARACTERS} characters and this one is {length}. Shorten it.")
    })
}
