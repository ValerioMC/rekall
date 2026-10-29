use serde::Serialize;

use crate::claude::{MemoryGroup, ResourceSummary};

/// Everything in the Claude folder the console edits, grouped the way it is shown.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceCatalog {
    pub root: String,
    pub instructions: Option<ResourceSummary>,
    pub skills: Vec<ResourceSummary>,
    pub commands: Vec<ResourceSummary>,
    pub agents: Vec<ResourceSummary>,
    pub memory_groups: Vec<MemoryGroup>,
}
