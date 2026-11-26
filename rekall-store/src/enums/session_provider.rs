use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

string_enum!(SessionProvider { Claude = "CLAUDE", Codex = "CODEX" });

#[allow(clippy::derivable_impls)]
impl Default for SessionProvider {
    fn default() -> Self {
        Self::Claude
    }
}

impl SessionProvider {
    pub fn label(self) -> &'static str {
        match self {
            Self::Claude => "Claude Code",
            Self::Codex => "Codex",
        }
    }
}
