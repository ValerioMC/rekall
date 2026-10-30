use std::path::PathBuf;

use rekall_core::claude::{EFFORT_LEVELS, MODEL_ALIASES};
use rekall_kernel::jstr;

/// How a session is started, with the model and effort already narrowed to the offered values.
#[derive(Clone, Debug, Default)]
pub struct ClaudeOptions {
    pub skip_permissions: bool,
    pub model: Option<String>,
    pub effort: Option<String>,
    /// A credential session's settings, whose deny rules guard its helpers.
    pub settings: Option<PathBuf>,
}

impl ClaudeOptions {
    pub fn chosen(skip_permissions: bool, model: Option<&str>, effort: Option<&str>) -> Self {
        Self { skip_permissions, model: normalise(model, &MODEL_ALIASES), effort: normalise(effort, &EFFORT_LEVELS), settings: None }
    }
}

fn normalise(value: Option<&str>, allowed: &[&str]) -> Option<String> {
    let trimmed = jstr::strip(value?).to_lowercase();
    allowed.contains(&trimmed.as_str()).then_some(trimmed)
}

#[cfg(test)]
#[path = "../../tests/unit/session/claude_options_tests.rs"]
mod tests;
