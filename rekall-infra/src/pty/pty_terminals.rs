use std::sync::Arc;

use async_trait::async_trait;
use rekall_core::claude::{TerminalHost, TerminalListener, TerminalOpening, TerminalPipe, TerminalView};
use rekall_kernel::{Id, Result};

use super::PtyTerminalManager;

/// The PTY terminals as the ports the console reaches them through.
#[derive(Clone)]
pub struct PtyTerminals(Arc<PtyTerminalManager>);

impl PtyTerminals {
    pub fn new(manager: Arc<PtyTerminalManager>) -> Self {
        Self(manager)
    }
}

#[async_trait]
impl TerminalHost for PtyTerminals {
    async fn open(&self, opening: TerminalOpening) -> Result<TerminalView> {
        self.0
            .open(opening.target, opening.step_id, opening.skip_permissions, opening.model.as_deref(), opening.effort.as_deref(), opening.mode)
            .await
    }

    fn list(&self) -> Vec<TerminalView> {
        self.0.list()
    }

    fn get(&self, id: Id) -> Option<TerminalView> {
        self.0.get(id)
    }

    async fn close(&self, id: Id, reason: &str) -> Option<TerminalView> {
        self.0.close(id, reason).await
    }
}

impl TerminalPipe for PtyTerminals {
    fn attach(&self, id: Id, listener: Arc<dyn TerminalListener>) -> Result<(TerminalView, u64)> {
        self.0.attach(id, listener)
    }

    fn detach(&self, id: Id, token: u64) {
        self.0.detach(id, token)
    }

    fn write(&self, id: Id, data: &[u8]) -> Result<()> {
        self.0.write(id, data)
    }

    fn resize(&self, id: Id, columns: i64, rows: i64) {
        self.0.resize(id, columns, rows)
    }
}
