use rekall_kernel::{RekallError, Result};
use serde::{Deserialize, Serialize};

use crate::claude::{CodexSessionOptions, SessionProvider};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct QueueLaunch {
    pub provider: SessionProvider,
    pub codex_options: Option<CodexSessionOptions>,
}

impl QueueLaunch {
    pub(in crate::queue) fn into_codex_options(self) -> Result<Option<CodexSessionOptions>> {
        if self.provider == SessionProvider::Claude {
            if self.codex_options.is_some() {
                return Err(RekallError::illegal("Codex options require the Codex provider."));
            }
            return Ok(None);
        }
        let options = self.codex_options.unwrap_or_default();
        CodexSessionOptions::new(options.skip_permissions, options.model.as_deref(), options.effort.as_deref()).map(Some)
    }
}
