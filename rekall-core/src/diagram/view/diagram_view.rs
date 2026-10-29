use serde::Serialize;

use crate::diagram::{DiagramSheetView, DiagramSummaryView};

/// A diagram in full: its summary and its sheets.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagramView {
    #[serde(flatten)]
    pub summary: DiagramSummaryView,
    pub sheets: Vec<DiagramSheetView>,
}
