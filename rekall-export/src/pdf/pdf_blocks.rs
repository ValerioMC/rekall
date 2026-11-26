use crate::markdown::{Block, InlineRun, ListItem};
use crate::style::DocumentStyle;
use crate::ExportError;

use super::base_style::BaseStyle;
use super::flow::Flow;
use super::flow_line::FlowLine;
use super::line_painter::LinePainter;
use super::pdf_faces::PdfFaces;
use super::pdf_font_kind::FontKind;
use super::pdf_metrics::PdfMetrics;
use super::placed::Placed;
use super::sheet::Sheet;
use super::text_style::TextStyle;

const HEADING_LEADING: f32 = 1.25;
const MARKER_GAP: f32 = 4.0;

/// Lays the blocks of every part out on pages. The code, table and figure layouts add their
/// methods from their own files.
pub(super) struct PdfBlocks<'a> {
    pub(super) faces: &'a PdfFaces,
    pub(super) flow: Flow<'a>,
    pub(super) painter: LinePainter<'a>,
    pub(super) sheet: Sheet,
    pub(super) pictures: Vec<&'a [u8]>,
}

impl<'a> PdfBlocks<'a> {
    pub(super) fn new(faces: &'a PdfFaces) -> Self {
        Self { faces, flow: Flow::new(faces), painter: LinePainter::new(faces), sheet: Sheet::new(), pictures: Vec::new() }
    }

    pub(super) fn finish(self) -> (Vec<Vec<Placed>>, Vec<&'a [u8]>) {
        (self.sheet.into_pages(), self.pictures)
    }

    pub(super) fn render_part(&mut self, blocks: &'a [Block], first: bool) -> Result<(), ExportError> {
        self.sheet.start_part(first);
        self.render_all(blocks, PdfMetrics::margin(), PdfMetrics::content_width(), false)
    }

    pub(super) fn body_base() -> BaseStyle {
        BaseStyle { size: DocumentStyle::BODY_POINTS as f32, bold: false, color: DocumentStyle::TEXT }
    }

    pub(super) fn body_line_height() -> f32 {
        DocumentStyle::BODY_POINTS as f32 * PdfMetrics::LEADING
    }

    pub(super) fn draw_lines(&mut self, lines: &[FlowLine], x: f32, leading: f32) {
        for line in lines {
            let height = line.size.max(DocumentStyle::BODY_POINTS as f32) * leading;
            self.sheet.ensure(height);
            let top = self.sheet.cursor();
            self.painter.draw(&mut self.sheet, line, x, top, height);
            self.sheet.advance(height);
        }
    }

    fn render_all(&mut self, blocks: &'a [Block], x: f32, width: f32, tight: bool) -> Result<(), ExportError> {
        for block in blocks {
            self.render(block, x, width, tight)?;
        }
        Ok(())
    }

    fn render(&mut self, block: &'a Block, x: f32, width: f32, tight: bool) -> Result<(), ExportError> {
        match block {
            Block::Heading { level, inlines } => self.heading(*level, inlines, x, width),
            Block::Paragraph(inlines) => self.paragraph(inlines, x, width, tight),
            Block::List { ordered, items } => return self.list(*ordered, items, x, width),
            Block::CodeBlock { text, .. } => self.code_block(text, x, width),
            Block::Table { header, rows } => self.table(header, rows, x, width),
            Block::Quote(blocks) => return self.quote(blocks, x, width),
            Block::Rule => self.rule(x, width),
            Block::Diagram { png } => return self.diagram(png, x, width),
            Block::DiagramFailed => self.error_box(DocumentStyle::DIAGRAM_ERROR_TEXT, x, width),
            Block::Picture(picture) => return self.picture(picture, x, width),
            Block::PictureMissing { description } => self.error_box(&DocumentStyle::missing_picture_text(description), x, width),
        }
        Ok(())
    }

    fn heading(&mut self, level: u8, inlines: &[InlineRun], x: f32, width: f32) {
        let index = usize::from(level.clamp(1, DocumentStyle::MAX_HEADING_LEVEL)) - 1;
        let size = DocumentStyle::HEADING_POINTS[index] as f32;
        let base = BaseStyle { size, bold: true, color: DocumentStyle::TEXT };
        let lines = self.flow.lines(&self.flow.items(inlines, &base), width);
        let line_height = size * HEADING_LEADING;
        let before = PdfMetrics::HEADING_BEFORE[index];
        let keep_with_next = Self::body_line_height() * PdfMetrics::KEEP_WITH_NEXT_LINES;
        self.sheet.ensure(before + line_height * lines.len() as f32 + keep_with_next);
        if !self.sheet.at_top() {
            self.sheet.advance(before);
        }
        self.draw_lines(&lines, x, HEADING_LEADING);
        self.sheet.advance(PdfMetrics::HEADING_AFTER);
    }

    fn paragraph(&mut self, inlines: &[InlineRun], x: f32, width: f32, tight: bool) {
        let lines = self.flow.lines(&self.flow.items(inlines, &Self::body_base()), width);
        if lines.is_empty() {
            return;
        }
        self.draw_lines(&lines, x, PdfMetrics::LEADING);
        self.sheet.advance(if tight { PdfMetrics::TIGHT_GAP } else { PdfMetrics::PARAGRAPH_GAP });
    }

    fn list(&mut self, ordered: bool, items: &'a [ListItem], x: f32, width: f32) -> Result<(), ExportError> {
        let text_x = x + PdfMetrics::LIST_INDENT;
        let text_width = width - PdfMetrics::LIST_INDENT;
        for (index, item) in items.iter().enumerate() {
            self.sheet.ensure(Self::first_height(item, text_width)?);
            self.draw_marker(ordered, index, item, x, text_x);
            self.render_all(&item.blocks, text_x, text_width, true)?;
        }
        self.sheet.advance(PdfMetrics::PARAGRAPH_GAP - PdfMetrics::TIGHT_GAP);
        Ok(())
    }

    /// What an item needs on the page of its marker: a line, or the whole picture it opens with.
    fn first_height(item: &ListItem, width: f32) -> Result<f32, ExportError> {
        match item.blocks.first() {
            Some(Block::Picture(picture)) => Ok(Self::picture_size(picture, width)?.1 + PdfMetrics::BLOCK_GAP),
            _ => Ok(Self::body_line_height()),
        }
    }

    fn draw_marker(&mut self, ordered: bool, index: usize, item: &ListItem, x: f32, text_x: f32) {
        let marker = match (item.task_marker(), ordered) {
            (Some(task), _) => task.trim_end().to_string(),
            (None, true) => format!("{}.", index + 1),
            (None, false) => "•".to_string(),
        };
        let style = TextStyle { kind: FontKind::Body, size: DocumentStyle::BODY_POINTS as f32, color: DocumentStyle::TEXT, link: None };
        let width = self.faces.face(FontKind::Body).width(&marker, style.size);
        let marker_x = if ordered || item.checked.is_some() { text_x - MARKER_GAP - width } else { x + MARKER_GAP };
        let top = self.sheet.cursor();
        self.painter.text(&mut self.sheet, marker_x, top, Self::body_line_height(), &marker, style);
    }

    fn quote(&mut self, blocks: &'a [Block], x: f32, width: f32) -> Result<(), ExportError> {
        self.sheet.ensure(Self::body_line_height() * PdfMetrics::KEEP_WITH_NEXT_LINES);
        let start_page = self.sheet.page_index();
        let start = self.sheet.cursor();
        self.render_all(blocks, x + PdfMetrics::QUOTE_INDENT, width - PdfMetrics::QUOTE_INDENT, false)?;
        let end_page = self.sheet.page_index();
        let end = self.sheet.cursor() - PdfMetrics::PARAGRAPH_GAP;
        for page in start_page..=end_page {
            let from = if page == start_page { start } else { Sheet::top() };
            let to = if page == end_page { end } else { Sheet::bottom() };
            if to > from {
                let rule_x = x + PdfMetrics::QUOTE_RULE_WIDTH / 2.0;
                let stroke = Placed::Stroke { x1: rule_x, y1: from, x2: rule_x, y2: to, width: PdfMetrics::QUOTE_RULE_WIDTH, color: DocumentStyle::QUOTE_RULE };
                self.sheet.push_on(page, stroke);
            }
        }
        Ok(())
    }

    fn rule(&mut self, x: f32, width: f32) {
        self.sheet.ensure(PdfMetrics::BLOCK_GAP * 2.0);
        let y = self.sheet.cursor() + PdfMetrics::BLOCK_GAP;
        self.sheet.push(Placed::Stroke { x1: x, y1: y, x2: x + width, y2: y, width: PdfMetrics::HAIRLINE_WIDTH, color: DocumentStyle::HAIRLINE });
        self.sheet.advance(PdfMetrics::BLOCK_GAP * 2.0);
    }
}
