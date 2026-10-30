use serde::{Deserialize, Serialize};

/// What a live session runs in: the interactive terminal, or the chat view over `claude` headless.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SessionKind {
    #[default]
    Terminal,
    Chat,
}

impl std::fmt::Display for SessionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SessionKind::Terminal => "TERMINAL",
            SessionKind::Chat => "CHAT",
        })
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/claude/model/session_kind_tests.rs"]
mod tests;
