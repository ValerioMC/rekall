use serde::Serialize;

use crate::graph::NodeKind;

/// One element a line of code belongs to, and how wide the span that matched is: the narrowest
/// span is the most specific answer.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceHit {
    pub node_id: String,
    pub kind: NodeKind,
    pub title: String,
    /// Lines in the matching span; `None` when the node claims the whole file.
    pub span_lines: Option<u32>,
}
