use rekall_kernel::{Id, Result};
use rekall_store::connection_access;
use rekall_store::repository as repo;

use super::CredentialAccessStreamEvent;
use crate::{in_write, Ctx};

/// The one write a terminal's credential endpoint makes: a line in a connection's access log. It
/// cannot read, change or delete a connection, and it never sees a credential.
#[derive(Clone)]
pub struct ConnectionAccessService {
    ctx: Ctx,
}

impl ConnectionAccessService {
    pub fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }

    pub async fn record_granted(&self, connection_id: Id, task_id: Option<Id>, terminal_id: Id) -> Result<()> {
        self.record(terminal_id, connection_access::Model::granted(connection_id, task_id, terminal_id.to_string(), self.ctx.now())).await
    }

    pub async fn record_refused(&self, connection_id: Id, task_id: Option<Id>, terminal_id: Id, reason: &str) -> Result<()> {
        self.record(terminal_id, connection_access::Model::refused(connection_id, task_id, terminal_id.to_string(), reason.to_string(), self.ctx.now())).await
    }

    /// The line and its event commit together: the pane never shows an access the log lacks.
    async fn record(&self, terminal_id: Id, access: connection_access::Model) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let event = CredentialAccessStreamEvent { terminal_id, connection_id: access.connection_id, granted: access.granted };
            repo::connection_access::record(tx.db(), access).await?;
            tx.publish(event);
            Ok(())
        })
    }
}
