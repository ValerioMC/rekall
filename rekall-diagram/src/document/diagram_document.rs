use serde::{Deserialize, Serialize};

use super::{DiagramKind, DiagramSheet, Scenario};

/// What one diagram holds: the scenario it follows and a sheet per view of the answer, in the order
/// they are shown. Diagrams stored before scenarios existed, and imported ones, have none.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiagramDocument {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scenario: Option<Scenario>,
    pub sheets: Vec<DiagramSheet>,
}

impl DiagramDocument {
    /// The type of every sheet, in order. A validated document declares one on each.
    pub fn kinds(&self) -> Vec<DiagramKind> {
        self.sheets.iter().filter_map(DiagramSheet::kind).collect()
    }

    /// Whether any sheet plays as a walkthrough, which is what the gallery marks.
    pub fn has_walkthrough(&self) -> bool {
        self.sheets.iter().any(|sheet| sheet.walkthrough.is_some())
    }
}
