use serde::{Deserialize, Serialize};

use super::{GraphEdge, GraphNode, RelationKind};

/// The whole document: a format marker and version, then the nodes and the edges.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SemanticGraph {
    pub format: String,
    pub version: u32,
    pub nodes: Vec<GraphNode>,
    #[serde(default)]
    pub edges: Vec<GraphEdge>,
}

impl SemanticGraph {
    pub fn node(&self, id: &str) -> Option<&GraphNode> {
        self.nodes.iter().find(|node| node.id == id)
    }

    /// The node that `contains` this one, if any. Validation guarantees at most one.
    pub fn container_of(&self, id: &str) -> Option<&str> {
        self.edges
            .iter()
            .find(|edge| edge.relation == RelationKind::Contains && edge.to == id)
            .map(|edge| edge.from.as_str())
    }
}
