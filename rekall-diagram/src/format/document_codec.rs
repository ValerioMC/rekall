use std::collections::BTreeMap;

use crate::document::{DiagramDocument, DiagramSheet, Walkthrough, WalkthroughMessage};

use super::DocumentFormatError;

/// Reads and writes the JSON form. Reading also normalises: titles, keys and walkthrough texts are
/// trimmed, a blank one is none, and the Mermaid loses the blank lines around it.
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
        let links: BTreeMap<String, _> = sheet.links.into_iter().map(|(key, sources)| (key.trim().to_string(), sources)).collect();
        DiagramSheet {
            title: present(sheet.title),
            mermaid: sheet.mermaid.trim_matches(['\n', '\r']).to_string(),
            links,
            walkthrough: sheet.walkthrough.map(Self::normalise_walkthrough),
        }
    }

    fn normalise_walkthrough(walkthrough: Walkthrough) -> Walkthrough {
        let messages = walkthrough
            .messages
            .into_iter()
            .map(|(key, message)| {
                let normalised = WalkthroughMessage {
                    explain: present(message.explain),
                    input: present(message.input),
                    output: present(message.output),
                    opens: message.opens,
                };
                (key.trim().to_string(), normalised)
            })
            .collect();
        Walkthrough { external: walkthrough.external.iter().map(|id| id.trim().to_string()).collect(), messages }
    }
}

/// Trimmed, and none when nothing is left.
fn present(text: Option<String>) -> Option<String> {
    text.map(|text| text.trim().to_string()).filter(|text| !text.is_empty())
}

#[cfg(test)]
#[path = "../../tests/unit/format/document_codec_tests.rs"]
mod tests;
