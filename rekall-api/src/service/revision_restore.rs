//! Writes a kept revision back as the current wrapup or description, through the same write path
//! as any other edit, so the text it replaces becomes a revision in turn. Here, beside the
//! catalog, because writing a task's description is a console write the MCP module must not have.

use rekall_common::{Id, Result};
use rekall_model::RevisionKind;
use rekall_service::revision::RestoredRevision;
use rekall_service::{in_write, Services};

use super::CatalogService;

#[derive(Clone)]
pub struct RevisionRestoreService {
    services: Services,
    catalog: CatalogService,
}

impl RevisionRestoreService {
    pub fn new(services: Services, catalog: CatalogService) -> Self {
        Self { services, catalog }
    }

    pub async fn restore(&self, task_id: Id, revision_id: Id) -> Result<RestoredRevision> {
        in_write!(&self.services.ctx, |tx| {
            let revision = self.services.revisions.find_in(&mut tx, task_id, revision_id).await?;
            match revision.kind {
                RevisionKind::Wrapup => {
                    self.services.wrapups.restore_in(&mut tx, task_id, &revision.body_markdown).await?;
                }
                RevisionKind::Description => {
                    self.catalog.restore_description_in(&mut tx, task_id, &revision.body_markdown).await?;
                }
            }
            Ok(RestoredRevision { task_id, kind: revision.kind, body_markdown: revision.body_markdown })
        })
    }
}
