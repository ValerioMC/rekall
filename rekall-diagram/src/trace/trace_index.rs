use crate::graph::{SemanticGraph, SourceLocation};

use super::TraceHit;

/// Answers which nodes of one graph cover a given line of a given file.
pub struct TraceIndex<'a> {
    graph: &'a SemanticGraph,
}

impl<'a> TraceIndex<'a> {
    pub fn of(graph: &'a SemanticGraph) -> Self {
        Self { graph }
    }

    /// The nodes covering `file:line`, narrowest span first; a whole-file claim sorts last.
    pub fn nodes_at(&self, file: &str, line: u32) -> Vec<TraceHit> {
        let mut hits: Vec<TraceHit> = self
            .graph
            .nodes
            .iter()
            .filter_map(|node| {
                let narrowest = node.sources.iter().filter(|source| source.covers(file, line)).map(Self::span_lines).min_by_key(|span| span.unwrap_or(u32::MAX))?;
                Some(TraceHit { node_id: node.id.clone(), kind: node.kind.clone(), title: node.title.clone(), span_lines: narrowest })
            })
            .collect();
        hits.sort_by_key(|hit| hit.span_lines.unwrap_or(u32::MAX));
        hits
    }

    fn span_lines(source: &SourceLocation) -> Option<u32> {
        let start = source.start_line?;
        Some(source.end_line.unwrap_or(start) - start + 1)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/trace/trace_index_tests.rs"]
mod tests;
