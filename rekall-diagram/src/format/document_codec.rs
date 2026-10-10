use std::collections::BTreeMap;

use crate::document::{DiagramDocument, DiagramSheet};

use super::DocumentFormatError;

/// Reads and writes the JSON form. Reading also normalises: titles and keys are trimmed, a blank
/// title is no title, and the Mermaid loses the blank lines around it.
pub struct DocumentCodec;

impl DocumentCodec {
    pub fn parse(text: &str) -> Result<DiagramDocument, DocumentFormatError> {
        let document: DiagramDocument = serde_json::from_str(text).map_err(|error| DocumentFormatError(error.to_string()))?;
        Ok(Self::normalise(document))
    }

    pub fn from_value(value: serde_json::Value) -> Result<DiagramDocument, DocumentFormatError> {
        let document: DiagramDocument = serde_json::from_value(value).map_err(|error| DocumentFormatError(error.to_string()))?;
        Ok(Self::normalise(document))
    }

    /// Compact JSON, the form a document is stored in.
    pub fn write(document: &DiagramDocument) -> String {
        serde_json::to_string(document).expect("a DiagramDocument always serialises")
    }

    fn normalise(document: DiagramDocument) -> DiagramDocument {
        DiagramDocument { sheets: document.sheets.into_iter().map(Self::normalise_sheet).collect() }
    }

    fn normalise_sheet(sheet: DiagramSheet) -> DiagramSheet {
        let title = sheet.title.map(|title| title.trim().to_string()).filter(|title| !title.is_empty());
        let links: BTreeMap<String, _> = sheet.links.into_iter().map(|(key, sources)| (key.trim().to_string(), sources)).collect();
        DiagramSheet { title, mermaid: sheet.mermaid.trim_matches(['\n', '\r']).to_string(), links }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/format/document_codec_tests.rs"]
mod tests;
