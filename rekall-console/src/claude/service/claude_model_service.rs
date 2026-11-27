use std::sync::Arc;

use rekall_core::claude::{ClaudeModel, ClaudeModelReader};
use rekall_kernel::Result;

/// The models the installed Claude Code offers, whether or not it knows Rekall. Nothing is kept,
/// so each read reflects the CLI as it is now.
#[derive(Clone)]
pub struct ClaudeModelService {
    reader: Arc<dyn ClaudeModelReader>,
}

impl ClaudeModelService {
    pub fn new(reader: Arc<dyn ClaudeModelReader>) -> Self {
        Self { reader }
    }

    pub async fn models(&self) -> Result<Vec<ClaudeModel>> {
        self.reader.models().await
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/claude/service/claude_model_service_tests.rs"]
mod tests;
