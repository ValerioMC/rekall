use serde::{Deserialize, Serialize};

use super::{Metadata, Provenance, RelationKind};

/// A typed relation from one node to another. `label` carries what the relation alone does not,
/// such as the condition on a `conditionally_leads_to`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GraphEdge {
    /// Optional on the way in; the codec numbers the edges that arrive without one.
    #[serde(default)]
    pub id: String,
    pub from: String,
    pub to: String,
    pub relation: RelationKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    #[serde(default, skip_serializing_if = "Metadata::is_empty")]
    pub metadata: Metadata,
}
