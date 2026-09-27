use rekall_diagram::SemanticGraph;
use serde::Serialize;

use super::DiagramSummaryView;

/// A diagram in full: its summary and the graph.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagramView {
    #[serde(flatten)]
    pub summary: DiagramSummaryView,
    pub graph: SemanticGraph,
}
