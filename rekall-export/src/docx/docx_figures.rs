use docx_rs::{AlignmentType, BorderType, Paragraph, ParagraphBorder, ParagraphBorderPosition, Pic, Run};

use crate::image::PngSize;
use crate::markdown::{PictureAlignment, PictureBlock};
use crate::style::DocumentStyle;
use crate::ExportError;

use super::docx_blocks::DocxBlocks;
use super::docx_element::DocxElement;
use super::docx_measure::DocxMeasure;

const ERROR_BORDER_EIGHTHS: usize = 8;

impl DocxBlocks {
    /// A diagram fills the text width and is centred in it.
    pub(super) fn diagram(png: &[u8]) -> Result<DocxElement, ExportError> {
        let (width, height) = PngSize::read(png)?.fit(DocumentStyle::TEXT_WIDTH_MM, DocumentStyle::FIGURE_MAX_HEIGHT_MM);
        Ok(DocxElement::paragraph(Self::figure(png, width, height, AlignmentType::Center)?))
    }

    pub(super) fn picture(&self, picture: &PictureBlock) -> Result<DocxElement, ExportError> {
        let (width, height) = picture.fit(DocumentStyle::TEXT_WIDTH_MM - self.inset_mm, DocumentStyle::FIGURE_MAX_HEIGHT_MM)?;
        let alignment = match picture.alignment {
            PictureAlignment::Left => AlignmentType::Left,
            PictureAlignment::Center => AlignmentType::Center,
            PictureAlignment::Right => AlignmentType::Right,
        };
        let paragraph = Self::figure(&picture.png, width, height, alignment)?;
        Ok(DocxElement::paragraph(paragraph.indent(Some(DocxMeasure::twips(self.inset_mm)), None, None, None)))
    }

    pub(super) fn error_box(text: &str) -> DocxElement {
        let run = Run::new().add_text(text).bold().color(DocumentStyle::ERROR_TEXT_COLOUR.hex());
        let paragraph = Paragraph::new().align(AlignmentType::Center).add_run(run);
        let boxed = Self::edit_property(paragraph, |property| {
            use ParagraphBorderPosition::{Bottom, Left, Right, Top};
            let bordered = [Top, Left, Bottom, Right].into_iter().fold(property, |property, position| {
                property.set_border(
                    ParagraphBorder::new(position).val(BorderType::Single).size(ERROR_BORDER_EIGHTHS).space(6).color(DocumentStyle::ERROR_BORDER.hex()),
                )
            });
            bordered.shading(Self::shading(DocumentStyle::ERROR_BACKGROUND))
        });
        DocxElement::paragraph(boxed)
    }

    fn figure(png: &[u8], width_mm: f64, height_mm: f64, alignment: AlignmentType) -> Result<Paragraph, ExportError> {
        let size = PngSize::read(png)?;
        let picture = Pic::new_with_dimensions(png.to_vec(), size.width, size.height).size(DocxMeasure::emu(width_mm), DocxMeasure::emu(height_mm));
        Ok(Paragraph::new().align(alignment).keep_lines(true).add_run(Run::new().add_image(picture)))
    }
}
