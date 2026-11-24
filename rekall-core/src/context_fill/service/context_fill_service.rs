use rekall_kernel::{Id, Result};
use rekall_store::context_fill;
use rekall_store::repository as repo;

use crate::chat::ContextFill;
use crate::context_fill::{ContextFillStreamEvent, ContextFillView};
use crate::{in_read, in_write, Ctx};

/// The one writer of `context_fill`. A turn on a step overwrites the step's row and the task's
/// own; a turn outside any step overwrites the task's.
#[derive(Clone)]
pub struct ContextFillService {
    ctx: Ctx,
}

impl ContextFillService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn record(&self, task_id: Id, step_id: Option<Id>, fill: ContextFill) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            if step_id.is_some() {
                repo::context_fill::upsert(tx.db(), self.row(task_id, step_id, fill)).await?;
            }
            repo::context_fill::upsert(tx.db(), self.row(task_id, None, fill)).await?;
            let fills = Self::views(&repo::context_fill::list_for_task(tx.db(), task_id).await?);
            tx.publish(ContextFillStreamEvent { task_id, fills });
            Ok(())
        })
    }

    pub async fn for_task(&self, task_id: Id) -> Result<Vec<ContextFillView>> {
        in_read!(&self.ctx, |tx| Ok(Self::views(&repo::context_fill::list_for_task(tx.db(), task_id).await?)))
    }

    fn row(&self, task_id: Id, step_id: Option<Id>, fill: ContextFill) -> context_fill::Model {
        context_fill::Model {
            id: Id::random(),
            task_id,
            step_id,
            used_tokens: i64::try_from(fill.used_tokens).unwrap_or(i64::MAX),
            window_tokens: i64::try_from(fill.window_tokens).unwrap_or(i64::MAX),
            recorded_at: self.ctx.now(),
        }
    }

    fn views(rows: &[context_fill::Model]) -> Vec<ContextFillView> {
        rows.iter().map(ContextFillView::of).collect()
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/context_fill/service/context_fill_service_tests.rs"]
mod tests;
