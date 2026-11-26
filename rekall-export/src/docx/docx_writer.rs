use std::io::Cursor;

use docx_rs::{AlignmentType, Docx, FieldCharType, Footer, Header, InstrPAGE, InstrText, Paragraph, Run};

use crate::document::ExportDocument;
use crate::markdown::MarkdownBlocks;
use crate::style::DocumentStyle;
use crate::ExportError;

use super::docx_blocks::DocxBlocks;
use super::docx_element::DocxElement;
use super::docx_measure::DocxMeasure;
use super::docx_styles::DocxStyles;

pub struct DocxWriter;

impl DocxWriter {
    pub fn write(document: &ExportDocument) -> Result<Vec<u8>, ExportError> {
        let mut blocks = DocxBlocks::new();
        let mut elements = Vec::new();
        for (index, part) in document.parts.iter().enumerate() {
            let read = MarkdownBlocks::read(part, &document.diagrams, &document.pictures);
            elements.extend(blocks.render_part(&read, index > 0)?);
        }
        let docx = DocxStyles::apply(Docx::new()).header(Self::header(&document.title)).footer(Self::footer());
        let docx = blocks.into_numberings().into_iter().fold(docx, Docx::add_numbering);
        let docx = elements.into_iter().fold(docx, |docx, element| match element {
            DocxElement::Paragraph(paragraph) => docx.add_paragraph(*paragraph),
            DocxElement::Table(table) => docx.add_table(*table),
        });
        Self::pack(docx)
    }

    fn pack(docx: Docx) -> Result<Vec<u8>, ExportError> {
        let mut buffer = Cursor::new(Vec::new());
        docx.build().pack(&mut buffer).map_err(|failure| ExportError::Docx(failure.to_string()))?;
        Ok(buffer.into_inner())
    }

    fn header(title: &str) -> Header {
        Header::new().add_paragraph(Self::small_paragraph(AlignmentType::Left).add_run(Self::small_run(title)))
    }

    fn footer() -> Footer {
        Footer::new().add_paragraph(Self::small_paragraph(AlignmentType::Center).add_run(Self::page_number_run()))
    }

    fn small_paragraph(alignment: AlignmentType) -> Paragraph {
        Paragraph::new().align(alignment).size(DocxMeasure::half_points(DocumentStyle::FURNITURE_POINTS)).color(DocumentStyle::MUTED.hex())
    }

    fn small_run(text: &str) -> Run {
        Self::furniture(Run::new().add_text(text))
    }

    fn page_number_run() -> Run {
        let field = Run::new()
            .add_field_char(FieldCharType::Begin, false)
            .add_instr_text(InstrText::PAGE(InstrPAGE::default()))
            .add_field_char(FieldCharType::Separate, false)
            .add_text("1")
            .add_field_char(FieldCharType::End, false);
        Self::furniture(field)
    }

    fn furniture(run: Run) -> Run {
        run.size(DocxMeasure::half_points(DocumentStyle::FURNITURE_POINTS)).color(DocumentStyle::MUTED.hex())
    }
}

#[cfg(test)]
#[path = "../../tests/unit/docx/docx_writer_tests.rs"]
mod tests;
