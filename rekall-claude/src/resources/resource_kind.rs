use serde::Serialize;

/// What a file in Claude Code's home folder is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ResourceKind {
    Instructions,
    Skill,
    Command,
    Agent,
    Memory,
    MemoryIndex,
}

impl ResourceKind {
    /// The index and the global instructions are edited but never deleted.
    pub fn is_deletable(self) -> bool {
        !matches!(self, Self::Instructions | Self::MemoryIndex)
    }
}
