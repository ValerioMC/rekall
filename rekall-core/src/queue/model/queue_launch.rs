use rekall_kernel::{RekallError, Result};
use serde::{Deserialize, Serialize};

use crate::claude::{CodexSessionOptions, SessionProvider};
use crate::queue::ClaudeQueueOptions;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct QueueLaunch {
    pub provider: SessionProvider,
    pub claude_options: Option<ClaudeQueueOptions>,
    pub codex_options: Option<CodexSessionOptions>,
}

impl QueueLaunch {
    pub(in crate::queue) fn validated(mut self) -> Result<Self> {
        match self.provider {
            SessionProvider::Claude if self.codex_options.is_some() => {
                return Err(RekallError::illegal("Codex options require the Codex provider."));
            }
            SessionProvider::Codex if self.claude_options.is_some() => {
                return Err(RekallError::illegal("Claude Code options require the Claude Code provider."));
            }
            SessionProvider::Claude => {
                self.claude_options = self
                    .claude_options
                    .map(|options| ClaudeQueueOptions::new(options.model.as_deref(), options.effort.as_deref()))
                    .transpose()?;
            }
            SessionProvider::Codex => {
                let options = self.codex_options.unwrap_or_default();
                self.codex_options = Some(CodexSessionOptions::new(
                    options.skip_permissions,
                    options.model.as_deref(),
                    options.effort.as_deref(),
                )?);
            }
        }
        Ok(self)
    }
}
