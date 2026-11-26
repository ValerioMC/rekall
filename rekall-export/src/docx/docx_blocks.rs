use docx_rs::{
    BorderType, IndentLevel, LineSpacing, Numbering, NumberingId, Paragraph, ParagraphBorder, ParagraphBorderPosition, ParagraphProperty, Run, Shading,
    ShdType, Table, TableBorder, TableBorderPosition, TableBorders, TableCell, TableRow, WidthType,
};

use crate::markdown::{Block, InlineRun, ListItem};
use crate::style::{DocumentStyle, Rgb};
use crate::ExportError;

use super::docx_element::DocxElement;
use super::docx_inlines::DocxInlines;
use super::docx_measure::DocxMeasure;
use super::docx_styles::{DocxStyles, BULLET_ABSTRACT_ID, DECIMAL_ABSTRACT_ID, LIST_LEVELS};

const HAIRLINE_EIGHTHS: usize = 4;
const QUOTE_RULE_EIGHTHS: usize = 18;
const QUOTE_INDENT_MM: f64 = 6.0;

/// Turns blocks into Word elements. Every list gets a numbering of its own so that a numbered
/// list starts at 1, which is why it keeps the numberings it created. `inset_mm` is how far the
/// list or quote being written stands in from the margin.
pub(super) struct DocxBlocks {
    numberings: Vec<Numbering>,
    pub(super) inset_mm: f64,
}

impl DocxBlocks {
    pub(super) fn new() -> Self {
        Self { numberings: Vec::new(), inset_mm: 0.0 }
    }

    pub(super) fn into_numberings(self) -> Vec<Numbering> {
        self.numberings
    }

    pub(super) fn render_part(&mut self, blocks: &[Block], starts_page: bool) -> Result<Vec<DocxElement>, ExportError> {
        let mut elements = self.render_all(blocks, 0)?;
        if starts_page {
            if let Some(DocxElement::Paragraph(first)) = elements.first_mut() {
                **first = std::mem::take(&mut **first).page_break_before(true);
            }
        }
        Ok(elements)
    }

    fn render_all(&mut self, blocks: &[Block], depth: usize) -> Result<Vec<DocxElement>, ExportError> {
        let mut elements = Vec::new();
        for block in blocks {
            elements.extend(self.render(block, depth)?);
        }
        Ok(elements)
    }

    fn render(&mut self, block: &Block, depth: usize) -> Result<Vec<DocxElement>, ExportError> {
        match block {
            Block::Heading { level, inlines } => Ok(vec![Self::heading(*level, inlines)]),
            Block::Paragraph(inlines) => Ok(vec![DocxElement::paragraph(DocxInlines::fill(Paragraph::new(), inlines, false))]),
            Block::List { ordered, items } => self.list(*ordered, items, depth),
            Block::CodeBlock { text, .. } => Ok(vec![Self::code_block(text)]),
            Block::Table { header, rows } => Ok(Self::table(header, rows).into_iter().collect()),
            Block::Quote(blocks) => Ok(self.quote(blocks, depth)?),
            Block::Rule => Ok(vec![Self::rule()]),
            Block::Diagram { png } => Ok(vec![Self::diagram(png)?]),
            Block::DiagramFailed => Ok(vec![Self::error_box(DocumentStyle::DIAGRAM_ERROR_TEXT)]),
            Block::Picture(picture) => Ok(vec![self.picture(picture)?]),
            Block::PictureMissing { description } => Ok(vec![Self::error_box(&DocumentStyle::missing_picture_text(description))]),
        }
    }

    fn heading(level: u8, inlines: &[InlineRun]) -> DocxElement {
        let paragraph = Paragraph::new().style(&format!("Heading{level}")).keep_next(true);
        DocxElement::paragraph(DocxInlines::fill(paragraph, inlines, false))
    }

    fn list(&mut self, ordered: bool, items: &[ListItem], depth: usize) -> Result<Vec<DocxElement>, ExportError> {
        let numbering_id = self.new_numbering(ordered);
        let level = depth.min(LIST_LEVELS - 1);
        let mut elements = Vec::new();
        for item in items {
            let mut rest = item.blocks.as_slice();
            let mut paragraph = Paragraph::new().numbering(NumberingId::new(numbering_id), IndentLevel::new(level));
            if let Some(marker) = item.task_marker() {
                paragraph = paragraph.add_run(Run::new().add_text(marker));
            }
            if let Some((Block::Paragraph(inlines), after)) = rest.split_first() {
                paragraph = DocxInlines::fill(paragraph, inlines, false);
                rest = after;
            }
            elements.push(DocxElement::paragraph(paragraph));
            elements.extend(self.list_item_rest(rest, depth)?);
        }
        Ok(elements)
    }

    fn list_item_rest(&mut self, blocks: &[Block], depth: usize) -> Result<Vec<DocxElement>, ExportError> {
        let mut elements = Vec::new();
        for block in blocks {
            match block {
                Block::List { ordered, items } => elements.extend(self.list(*ordered, items, depth + 1)?),
                Block::Paragraph(inlines) => {
                    let paragraph = Paragraph::new().indent(Some(DocxStyles::list_indent(depth)), None, None, None);
                    elements.push(DocxElement::paragraph(DocxInlines::fill(paragraph, inlines, false)));
                }
                other => elements.extend(self.within(DocxStyles::list_indent_mm(depth), |inner| inner.render(other, depth))?),
            }
        }
        Ok(elements)
    }

    fn within<T>(&mut self, inset_mm: f64, write: impl FnOnce(&mut Self) -> T) -> T {
        self.inset_mm += inset_mm;
        let written = write(self);
        self.inset_mm -= inset_mm;
        written
    }

    fn new_numbering(&mut self, ordered: bool) -> usize {
        let id = self.numberings.len() + 1;
        let abstract_id = if ordered { DECIMAL_ABSTRACT_ID } else { BULLET_ABSTRACT_ID };
        self.numberings.push(Numbering::new(id, abstract_id));
        id
    }

    fn code_block(text: &str) -> DocxElement {
        let run = DocxInlines::text_run(text).fonts(DocxInlines::code_fonts()).size(DocxMeasure::half_points(DocumentStyle::CODE_POINTS));
        let paragraph = Paragraph::new().add_run(run);
        DocxElement::paragraph(Self::edit_property(paragraph, |property| property.shading(Self::shading(DocumentStyle::CODE_BACKGROUND))))
    }

    fn table(header: &[Vec<InlineRun>], rows: &[Vec<Vec<InlineRun>>]) -> Option<DocxElement> {
        let columns = header.len();
        if columns == 0 {
            return None;
        }
        let table_width = DocxMeasure::twips(DocumentStyle::TEXT_WIDTH_MM) as usize;
        let column_width = table_width / columns;
        let header_row = Self::table_row(header, columns, column_width, true);
        let body_rows = rows.iter().map(|row| Self::table_row(row, columns, column_width, false));
        let table = Table::new(std::iter::once(header_row).chain(body_rows).collect())
            .set_grid(vec![column_width; columns])
            .width(table_width, WidthType::Dxa)
            .set_borders(Self::hairline_borders());
        Some(DocxElement::table(table))
    }

    fn table_row(cells: &[Vec<InlineRun>], columns: usize, column_width: usize, header: bool) -> TableRow {
        let empty: Vec<InlineRun> = Vec::new();
        let cells = (0..columns)
            .map(|column| {
                let inlines = cells.get(column).unwrap_or(&empty);
                let paragraph = DocxInlines::fill(Paragraph::new().line_spacing(LineSpacing::new().before(40).after(40)), inlines, header);
                let cell = TableCell::new().width(column_width, WidthType::Dxa).add_paragraph(paragraph);
                if header {
                    cell.shading(Self::shading(DocumentStyle::CODE_BACKGROUND))
                } else {
                    cell
                }
            })
            .collect();
        TableRow::new(cells)
    }

    fn hairline_borders() -> TableBorders {
        use TableBorderPosition::{Bottom, InsideH, InsideV, Left, Right, Top};
        [Top, Left, Bottom, Right, InsideH, InsideV].into_iter().fold(TableBorders::with_empty(), |borders, position| {
            borders.set(TableBorder::new(position).border_type(BorderType::Single).size(HAIRLINE_EIGHTHS).color(DocumentStyle::HAIRLINE.hex()))
        })
    }

    fn quote(&mut self, blocks: &[Block], depth: usize) -> Result<Vec<DocxElement>, ExportError> {
        let inner = self.within(QUOTE_INDENT_MM, |inner| inner.render_all(blocks, depth))?;
        Ok(inner
            .into_iter()
            .map(|element| match element {
                DocxElement::Paragraph(paragraph) => DocxElement::paragraph(Self::quoted(*paragraph)),
                table => table,
            })
            .collect())
    }

    fn quoted(paragraph: Paragraph) -> Paragraph {
        let rule = ParagraphBorder::new(ParagraphBorderPosition::Left)
            .val(BorderType::Single)
            .size(QUOTE_RULE_EIGHTHS)
            .space(8)
            .color(DocumentStyle::QUOTE_RULE.hex());
        let ruled = Self::edit_property(paragraph, |property| property.set_border(rule));
        if ruled.has_numbering {
            ruled
        } else {
            ruled.indent(Some(DocxMeasure::twips(QUOTE_INDENT_MM)), None, None, None)
        }
    }

    fn rule() -> DocxElement {
        let line = ParagraphBorder::new(ParagraphBorderPosition::Bottom).val(BorderType::Single).size(HAIRLINE_EIGHTHS).space(1).color(DocumentStyle::HAIRLINE.hex());
        DocxElement::paragraph(Self::edit_property(Paragraph::new(), |property| property.set_border(line)))
    }

    pub(super) fn shading(fill: Rgb) -> Shading {
        Shading::new().shd_type(ShdType::Clear).color("auto").fill(fill.hex())
    }

    pub(super) fn edit_property(mut paragraph: Paragraph, edit: impl FnOnce(ParagraphProperty) -> ParagraphProperty) -> Paragraph {
        paragraph.property = edit(std::mem::take(&mut paragraph.property));
        paragraph
    }
}
