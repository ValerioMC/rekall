use serde::Serialize;

/// What a document that already says a memory is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CoveringKind {
    GlobalInstructions,
    ProjectInstructions,
    Skill,
}

/// A `CLAUDE.md` or a skill that already holds what a memory says.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoveringDocument {
    pub kind: CoveringKind,
    pub label: String,
    /// Where it is on disk, for showing.
    pub location: String,
    /// Its path under `~/.claude` when the console can open it; a project's `CLAUDE.md` is not.
    pub resource_path: Option<String>,
}
