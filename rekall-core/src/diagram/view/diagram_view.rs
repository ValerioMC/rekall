use rekall_diagram::Scenario;
use serde::Serialize;

use crate::diagram::{DiagramSheetView, DiagramSummaryView};

/// A diagram in full: its summary, the scenario it follows (none on an imported or older one) and its sheets.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagramView {
    #[serde(flatten)]
    pub summary: DiagramSummaryView,
    pub scenario: Option<Scenario>,
    pub sheets: Vec<DiagramSheetView>,
}
