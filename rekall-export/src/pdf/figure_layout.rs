use crate::image::PngSize;
use crate::markdown::{InlineRun, PictureBlock};
use crate::style::DocumentStyle;
use crate::ExportError;

use super::base_style::BaseStyle;
use super::pdf_blocks::PdfBlocks;
use super::pdf_metrics::PdfMetrics;
use super::placed::Placed;

const ERROR_TEXT_POINTS: f32 = 10.5;

impl<'a> PdfBlocks<'a> {
    /// A diagram fills the width it is given and is centred in it.
    pub(super) fn diagram(&mut self, png: &'a [u8], x: f32, width: f32) -> Result<(), ExportError> {
        let (fitted_width, fitted_height) = PngSize::read(png)?.fit(f64::from(width), f64::from(PdfMetrics::figure_max_height()));
        let (figure_width, figure_height) = (fitted_width as f32, fitted_height as f32);
        self.figure(png, x + (width - figure_width) / 2.0, figure_width, figure_height);
        Ok(())
    }

    pub(super) fn picture(&mut self, picture: &'a PictureBlock, x: f32, width: f32) -> Result<(), ExportError> {
        let (picture_width, picture_height) = Self::picture_size(picture, width)?;
        let left = x + (width - picture_width) * picture.alignment.share_before() as f32;
        self.figure(&picture.png, left, picture_width, picture_height);
        Ok(())
    }

    /// Its width and height in points when the page leaves it `width` points.
    pub(super) fn picture_size(picture: &PictureBlock, width: f32) -> Result<(f32, f32), ExportError> {
        let (width_mm, height_mm) = picture.fit(PdfMetrics::millimetres(width), DocumentStyle::FIGURE_MAX_HEIGHT_MM)?;
        Ok((PdfMetrics::points(width_mm), PdfMetrics::points(height_mm)))
    }

    pub(super) fn error_box(&mut self, text: &str, x: f32, width: f32) {
        let base = BaseStyle { size: ERROR_TEXT_POINTS, bold: true, color: DocumentStyle::ERROR_TEXT_COLOUR };
        let text_width = width - 2.0 * PdfMetrics::CELL_PAD_X;
        let lines = self.flow.lines(&self.flow.items(&[InlineRun::plain(text)], &base), text_width);
        let line_height = ERROR_TEXT_POINTS * PdfMetrics::LEADING;
        let height = PdfMetrics::ERROR_BOX_HEIGHT.max(line_height * lines.len() as f32 + 2.0 * PdfMetrics::CODE_PAD_Y);
        self.sheet.ensure(height + PdfMetrics::BLOCK_GAP);
        let top = self.sheet.cursor();
        self.sheet.push(Placed::Rect { x, top, width, height, fill: DocumentStyle::ERROR_BACKGROUND });
        self.error_border(x, top, width, height);
        let first_line_top = top + (height - line_height * lines.len() as f32) / 2.0;
        for (index, line) in lines.iter().enumerate() {
            let left = x + (width - line.width) / 2.0;
            self.painter.draw(&mut self.sheet, line, left, first_line_top + line_height * index as f32, line_height);
        }
        self.sheet.advance(height + PdfMetrics::BLOCK_GAP);
    }

    fn figure(&mut self, png: &'a [u8], left: f32, width: f32, height: f32) {
        self.sheet.ensure(height + PdfMetrics::BLOCK_GAP);
        let picture = self.pictures.len();
        self.pictures.push(png);
        let top = self.sheet.cursor();
        self.sheet.push(Placed::Picture { picture, x: left, top, width, height });
        self.sheet.advance(height + PdfMetrics::BLOCK_GAP);
    }

    fn error_border(&mut self, x: f32, top: f32, width: f32, height: f32) {
        let (right, bottom) = (x + width, top + height);
        for (x1, y1, x2, y2) in [(x, top, right, top), (right, top, right, bottom), (right, bottom, x, bottom), (x, bottom, x, top)] {
            self.sheet.push(Placed::Stroke { x1, y1, x2, y2, width: PdfMetrics::ERROR_BORDER_WIDTH, color: DocumentStyle::ERROR_BORDER });
        }
    }
}
