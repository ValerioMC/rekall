use serde::Serialize;

use crate::claude::{MemoryRef, PromotionTarget};

/// Memories in several projects that say the same thing.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromotionCandidate {
    /// The words the memories share, most common first.
    pub topic: Vec<String>,
    pub memories: Vec<MemoryRef>,
    pub group_count: usize,
    pub target: PromotionTarget,
    pub reason: String,
}
