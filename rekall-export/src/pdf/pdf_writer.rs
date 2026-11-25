use crate::document::ExportDocument;
use crate::markdown::{Block, MarkdownBlocks};
use crate::ExportError;

use super::pdf_blocks::PdfBlocks;
use super::pdf_emitter::PdfEmitter;
use super::pdf_faces::PdfFaces;

pub struct PdfWriter;

impl PdfWriter {
    pub fn write(document: &ExportDocument) -> Result<Vec<u8>, ExportError> {
        let faces = PdfFaces::load()?;
        let parts: Vec<Vec<Block>> = document.parts.iter().map(|part| MarkdownBlocks::read(part, &document.diagrams)).collect();
        let mut blocks = PdfBlocks::new(&faces);
        for (index, part) in parts.iter().enumerate() {
            blocks.render_part(part, index == 0)?;
        }
        let (pages, pictures) = blocks.finish();
        PdfEmitter::new(&faces, &document.title).emit(pages, &pictures)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/pdf/pdf_writer_tests.rs"]
mod tests;
