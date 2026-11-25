use rekall_kernel::{RekallError, Result};
use rekall_store::repository as repo;
use rekall_store::task::{Model, DESCRIPTION_MAX_CHARACTERS};

use crate::task::{DescriptionReplaced, TaskStreamEvent};
use crate::{in_write, load, text_rule, Ctx};

/// Replaces one task's description for a session, rewriting or discussing it, and announces a changed
/// one. It does not go through the console's revision tracking: such a write is not a change the
/// person asked to have worked, so it opens no revision.
#[derive(Clone)]
pub struct TaskDescriptionService {
    ctx: Ctx,
}

impl TaskDescriptionService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    /// The old text is overwritten and nothing keeps it. An open console must read the new one, or
    /// its next task update would overwrite it.
    pub async fn replace(&self, project_label: Option<&str>, task_label: &str, body: Option<&str>) -> Result<DescriptionReplaced> {
        in_write!(&self.ctx, |tx| {
            let before = load::resolve_task(tx.db(), project_label, task_label).await?;
            let after = Model { description: Some(validated_body(body)?), ..before.clone() };
            let changed = after != before;
            repo::update_if_changed(tx.db(), &before, after, self.ctx.now()).await?;
            if changed {
                tx.publish(TaskStreamEvent { task_id: before.id, project_id: before.project_id });
            }
            Ok(DescriptionReplaced { task_anchor: before.anchor(), changed })
        })
    }
}

fn validated_body(body: Option<&str>) -> Result<String> {
    let text = text_rule::present(body).ok_or_else(|| RekallError::illegal("A description needs a body: the whole new text of the task. Nothing was written."))?;
    text_rule::within(text, DESCRIPTION_MAX_CHARACTERS, |length| {
        format!("A description is capped at {DESCRIPTION_MAX_CHARACTERS} characters and this one is {length}. Shorten it. Nothing was written.")
    })
}
