use serde::Serialize;

use super::{CoveringDocument, MemoryRef};

/// A memory whose content is already in a `CLAUDE.md` or a skill Claude loads anyway.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsoleteMemory {
    pub memory: MemoryRef,
    pub covered_by: CoveringDocument,
    /// How much of the memory's wording the document repeats, 0 to 100.
    pub overlap_percent: u8,
    pub shared_phrases: Vec<String>,
}
