use serde::Serialize;

use super::ResourceSummary;

/// A file opened for editing.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceFile {
    pub summary: ResourceSummary,
    pub body: String,
}
