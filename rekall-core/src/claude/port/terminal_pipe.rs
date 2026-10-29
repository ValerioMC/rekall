use std::sync::Arc;

use rekall_kernel::{Id, Result};

use crate::claude::{TerminalListener, TerminalView};

/// The bytes of one live terminal: a pane attaches to hear its output and writes its input.
pub trait TerminalPipe: Send + Sync {
    /// Replays the scrollback to `listener`, then follows live output; the token detaches it.
    fn attach(&self, id: Id, listener: Arc<dyn TerminalListener>) -> Result<(TerminalView, u64)>;

    fn detach(&self, id: Id, token: u64);

    fn write(&self, id: Id, data: &[u8]) -> Result<()>;

    fn resize(&self, id: Id, columns: i64, rows: i64);
}
