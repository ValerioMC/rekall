use serde::Serialize;

/// `reference`: whether the part is a note handed over as a reference rather than in full.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextSizePart {
    pub label: String,
    pub characters: usize,
    pub reference: bool,
}
