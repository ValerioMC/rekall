use std::collections::BTreeMap;

use rekall_diagram::{DiagramKind, DiagramSheet, SourceLocation};
use serde::Serialize;

/// A sheet as the console reads it: the stored sheet with the type its Mermaid declares.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagramSheetView {
    pub kind: DiagramKind,
    pub title: Option<String>,
    pub mermaid: String,
    pub links: BTreeMap<String, Vec<SourceLocation>>,
}

impl DiagramSheetView {
    /// None for a sheet that declares no known type, which validation never lets through.
    pub fn of(sheet: &DiagramSheet) -> Option<Self> {
        Some(Self { kind: sheet.kind()?, title: sheet.title.clone(), mermaid: sheet.mermaid.clone(), links: sheet.links.clone() })
    }
}
