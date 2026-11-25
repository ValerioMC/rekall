use crate::markdown::MarkdownBlocks;

use super::{DiagramImage, ExportPart};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportDocument {
    pub title: String,
    pub parts: Vec<ExportPart>,
    pub diagrams: Vec<DiagramImage>,
}

impl ExportDocument {
    pub fn diagrams_missing(&self) -> usize {
        self.parts
            .iter()
            .flat_map(|part| MarkdownBlocks::mermaid_sources(&part.markdown))
            .filter(|source| !self.diagrams.iter().any(|image| image.source.trim() == source))
            .count()
    }
}

#[cfg(test)]
#[path = "../../tests/unit/document/export_document_tests.rs"]
mod tests;
