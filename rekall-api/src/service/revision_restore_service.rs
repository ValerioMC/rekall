//! Writes a kept revision back as the current wrapup or description, through the same write path
//! as any other edit, so the text it replaces becomes a revision in turn. Here, beside
//! `TaskService`, because writing a task's description is a console write the MCP module must not have.

use rekall_common::{Id, Result};
use rekall_model::RevisionKind;
use rekall_service::revision::{RestoredRevision, TaskRevisionService};
use rekall_service::wrapup::WrapupService;
use rekall_service::{in_write, Ctx};

use super::TaskService;

#[derive(Clone)]
pub struct RevisionRestoreService {
    ctx: Ctx,
    revisions: TaskRevisionService,
    wrapups: WrapupService,
    tasks: TaskService,
}

impl RevisionRestoreService {
    pub fn new(ctx: Ctx, revisions: TaskRevisionService, wrapups: WrapupService, tasks: TaskService) -> Self {
        Self { ctx, revisions, wrapups, tasks }
    }

    pub async fn restore(&self, task_id: Id, revision_id: Id) -> Result<RestoredRevision> {
        in_write!(&self.ctx, |tx| {
            let revision = self.revisions.find_in(&mut tx, task_id, revision_id).await?;
            match revision.kind {
                RevisionKind::Wrapup => {
                    self.wrapups.restore_in(&mut tx, task_id, &revision.body_markdown).await?;
                }
                RevisionKind::Description => {
                    self.tasks.restore_description_in(&mut tx, task_id, &revision.body_markdown).await?;
                }
            }
            Ok(RestoredRevision { task_id, kind: revision.kind, body_markdown: revision.body_markdown })
        })
    }
}
