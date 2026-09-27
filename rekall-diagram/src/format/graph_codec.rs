use std::collections::HashSet;

use crate::graph::SemanticGraph;

use super::GraphFormatError;

/// The marker every graph document carries in its `format` field.
pub const FORMAT: &str = "rekall.semantic-graph";

/// The only version this build reads and writes.
pub const CURRENT_VERSION: u32 = 1;

/// Reads and writes the JSON form. Reading also normalises: titles are trimmed and edges that
/// arrived without an id are numbered, so every stored edge can be addressed.
pub struct GraphCodec;

impl GraphCodec {
    pub fn parse(text: &str) -> Result<SemanticGraph, GraphFormatError> {
        let graph: SemanticGraph = serde_json::from_str(text).map_err(|error| GraphFormatError::Malformed(error.to_string()))?;
        Self::accept(graph)
    }

    pub fn from_value(value: serde_json::Value) -> Result<SemanticGraph, GraphFormatError> {
        let graph: SemanticGraph = serde_json::from_value(value).map_err(|error| GraphFormatError::Malformed(error.to_string()))?;
        Self::accept(graph)
    }

    /// Compact JSON, the form a graph is stored in.
    pub fn write(graph: &SemanticGraph) -> String {
        serde_json::to_string(graph).expect("a SemanticGraph always serialises")
    }

    fn accept(mut graph: SemanticGraph) -> Result<SemanticGraph, GraphFormatError> {
        if graph.format != FORMAT {
            return Err(GraphFormatError::UnknownFormat(graph.format));
        }
        if graph.version != CURRENT_VERSION {
            return Err(GraphFormatError::UnsupportedVersion(graph.version));
        }
        Self::normalise(&mut graph);
        Ok(graph)
    }

    fn normalise(graph: &mut SemanticGraph) {
        for node in &mut graph.nodes {
            node.id = node.id.trim().to_string();
            node.title = node.title.trim().to_string();
        }
        let mut taken: HashSet<String> = graph.edges.iter().map(|edge| edge.id.trim().to_string()).filter(|id| !id.is_empty()).collect();
        let mut next = 1;
        for edge in &mut graph.edges {
            edge.from = edge.from.trim().to_string();
            edge.to = edge.to.trim().to_string();
            edge.id = edge.id.trim().to_string();
            if edge.id.is_empty() {
                while taken.contains(&format!("e{next}")) {
                    next += 1;
                }
                edge.id = format!("e{next}");
                taken.insert(edge.id.clone());
            }
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/format/graph_codec_tests.rs"]
mod tests;
