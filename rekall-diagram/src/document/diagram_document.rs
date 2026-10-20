use serde::{Deserialize, Serialize};

use super::{DiagramKind, DiagramSheet};

/// What one diagram holds: a sheet per view of the answer, in the order they are shown.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiagramDocument {
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

    /// How many elements, across every sheet, are tied to code.
    pub fn linked_elements(&self) -> usize {
        self.sheets.iter().map(|sheet| sheet.links.len()).sum()
    }
}
