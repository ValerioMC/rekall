use serde::Serialize;

use crate::claude::ResourceSummary;

/// A memory named by a finding: enough to list it and to open it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryRef {
    pub path: String,
    pub name: String,
    pub group_id: String,
    pub memory_type: Option<String>,
}

impl MemoryRef {
    pub fn of(summary: &ResourceSummary, group_id: &str) -> Self {
        Self { path: summary.path.clone(), name: summary.name.clone(), group_id: group_id.to_string(), memory_type: summary.memory_type.clone() }
    }
}
