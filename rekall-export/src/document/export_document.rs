use std::collections::HashSet;

use crate::markdown::MarkdownBlocks;

use super::{DiagramImage, ExportPart, ExportPicture};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportDocument {
    pub title: String,
    pub parts: Vec<ExportPart>,
    pub diagrams: Vec<DiagramImage>,
    pub pictures: Vec<ExportPicture>,
}

impl ExportDocument {
    pub fn diagrams_missing(&self) -> usize {
        self.parts
            .iter()
            .flat_map(|part| MarkdownBlocks::mermaid_sources(&part.markdown))
            .filter(|source| !self.diagrams.iter().any(|image| image.source.trim() == source))
            .count()
    }

    /// The pictures a writer needs handed to it: every address not on the web, once each.
    pub fn picture_addresses(&self) -> Vec<String> {
        let mut seen = HashSet::new();
        self.parts
            .iter()
            .flat_map(|part| MarkdownBlocks::local_picture_addresses(&part.markdown))
            .filter(|address| seen.insert(address.clone()))
            .collect()
    }
}

#[cfg(test)]
#[path = "../../tests/unit/document/export_document_tests.rs"]
mod tests;
