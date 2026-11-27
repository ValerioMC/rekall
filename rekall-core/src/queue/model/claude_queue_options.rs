use rekall_kernel::Result;
use serde::{Deserialize, Serialize};

use crate::claude::{claude_effort_choice, claude_model_choice};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ClaudeQueueOptions {
    pub model: Option<String>,
    pub effort: Option<String>,
}

impl ClaudeQueueOptions {
    pub fn new(model: Option<&str>, effort: Option<&str>) -> Result<Self> {
        Ok(Self { model: claude_model_choice(model)?, effort: claude_effort_choice(effort)? })
    }
}
