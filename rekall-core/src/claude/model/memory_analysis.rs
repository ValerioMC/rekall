use serde::Serialize;

use crate::claude::{ObsoleteMemory, PromotionCandidate};

/// What reading every memory against the others and against the instructions found.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryAnalysis {
    pub memories_read: usize,
    pub documents_read: usize,
    pub promotions: Vec<PromotionCandidate>,
    pub obsolete: Vec<ObsoleteMemory>,
}
