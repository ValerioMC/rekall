use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(
    /// What a live session runs in: the interactive terminal, or the chat view over `claude` headless.
    SessionKind { Terminal = "TERMINAL", Chat = "CHAT" }
);

// `string_enum!` cannot mark a variant `#[default]`.
#[allow(clippy::derivable_impls)]
impl Default for SessionKind {
    fn default() -> Self {
        Self::Terminal
    }
}

#[cfg(test)]
#[path = "../../tests/unit/enums/session_kind_tests.rs"]
mod tests;
