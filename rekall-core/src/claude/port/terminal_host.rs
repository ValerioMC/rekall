use async_trait::async_trait;
use rekall_kernel::{Id, Result};
use tokio::sync::broadcast;

use crate::claude::{TerminalEnded, TerminalOpening, TerminalView};

/// The live `claude` sessions: opened on a task or project, listed, closed, and renewed.
#[async_trait]
pub trait TerminalHost: Send + Sync {
    async fn open(&self, opening: TerminalOpening) -> Result<TerminalView>;

    fn list(&self) -> Vec<TerminalView>;

    fn get(&self, id: Id) -> Option<TerminalView>;

    /// The session as it was when it closed, or none when no session has that id.
    async fn close(&self, id: Id, reason: &str) -> Option<TerminalView>;

    /// Every end of every session, however it came.
    fn subscribe_ended(&self) -> broadcast::Receiver<TerminalEnded>;

    /// True while some live session is on this task, whoever opened it.
    fn has_live_for(&self, task_id: Id) -> bool;

    /// The live session working this task's steps, if one is.
    fn work_session_for(&self, task_id: Id) -> Option<TerminalView>;

    /// Ends a session that claimed the last step of its group and opens another on the same task
    /// with the same choices. Only a session working a task's steps can be renewed.
    async fn renew(&self, id: Id) -> Result<TerminalView>;
}
