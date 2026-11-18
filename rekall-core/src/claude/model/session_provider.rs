use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SessionProvider {
    #[default]
    Claude,
    Codex,
}

impl SessionProvider {
    pub fn command_prompt(self, line: &str) -> String {
        match (self, line.strip_prefix("/rk ")) {
            (Self::Codex, Some(arguments)) => format!("Use the $rekall skill with this exact argument line: {arguments}"),
            _ => line.to_owned(),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Claude => "Claude Code",
            Self::Codex => "Codex",
        }
    }
}
