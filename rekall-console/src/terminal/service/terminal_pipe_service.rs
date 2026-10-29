use std::sync::Arc;

use rekall_core::claude::{TerminalListener, TerminalPipe, TerminalView};
use rekall_kernel::{Id, Result};

/// The bytes between one terminal and its pane. Nothing is decided here: the pipe answers.
#[derive(Clone)]
pub struct TerminalPipeService {
    pipe: Arc<dyn TerminalPipe>,
}

impl TerminalPipeService {
    pub fn new(pipe: Arc<dyn TerminalPipe>) -> Self {
        Self { pipe }
    }

    pub fn attach(&self, id: Id, listener: Arc<dyn TerminalListener>) -> Result<(TerminalView, u64)> {
        self.pipe.attach(id, listener)
    }

    pub fn detach(&self, id: Id, token: u64) {
        self.pipe.detach(id, token)
    }

    pub fn write(&self, id: Id, data: &[u8]) -> Result<()> {
        self.pipe.write(id, data)
    }

    pub fn resize(&self, id: Id, columns: i64, rows: i64) {
        self.pipe.resize(id, columns, rows)
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/terminal/service/terminal_pipe_service_tests.rs"]
mod tests;
