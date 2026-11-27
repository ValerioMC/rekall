use rekall_kernel::{jstr, RekallError, Result};
use serde::{Deserialize, Serialize};

use crate::claude::{EFFORT_LEVELS, MODEL_ALIASES};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ClaudeQueueOptions {
    pub model: Option<String>,
    pub effort: Option<String>,
}

impl ClaudeQueueOptions {
    pub fn new(model: Option<&str>, effort: Option<&str>) -> Result<Self> {
        Ok(Self {
            model: choice(model, &MODEL_ALIASES, "model")?,
            effort: choice(effort, &EFFORT_LEVELS, "effort")?,
        })
    }
}

fn choice(value: Option<&str>, allowed: &[&str], what: &str) -> Result<Option<String>> {
    let Some(value) = value else { return Ok(None) };
    let normalised = jstr::strip(value).to_lowercase();
    if normalised == "default" {
        return Ok(None);
    }
    if !allowed.contains(&normalised.as_str()) {
        return Err(RekallError::illegal(format!("'{value}' is not a {what} Claude Code accepts.")));
    }
    Ok(Some(normalised))
}
