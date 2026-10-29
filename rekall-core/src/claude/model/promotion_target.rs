use serde::Serialize;

/// Where a memory that keeps coming back belongs instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PromotionTarget {
    /// A short rule: one line in the global `CLAUDE.md`.
    Instructions,
    /// A procedure with steps or code: a skill Claude loads when it applies.
    Skill,
}
