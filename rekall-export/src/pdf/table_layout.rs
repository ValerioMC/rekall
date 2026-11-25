use crate::markdown::InlineRun;
use crate::style::DocumentStyle;

use super::base_style::BaseStyle;
use super::flow_item::FlowItem;
use super::flow_line::FlowLine;
use super::pdf_blocks::PdfBlocks;
use super::pdf_metrics::PdfMetrics;
use super::placed::Placed;

const TABLE_LEADING: f32 = 1.4;

type Cell = Vec<FlowItem>;

struct RowChunk {
    top: f32,
    offset: usize,
    taken: usize,
    line_height: f32,
}

struct TableRow {
    header: bool,
    cells: Vec<Cell>,
}

impl<'a> PdfBlocks<'a> {
    /// Columns share the width by what their text needs. A row that does not fit the page is split
    /// by lines, and every chunk of it is boxed on its own.
    pub(super) fn table(&mut self, header: &[Vec<InlineRun>], rows: &[Vec<Vec<InlineRun>>], x: f32, width: f32) {
        let columns = header.len();
        if columns == 0 {
            return;
        }
        let table_rows = self.table_rows(header, rows, columns);
        let widths = Self::column_widths(&table_rows, columns, width);
        for row in &table_rows {
            self.table_row(row, &widths, x);
        }
        self.sheet.advance(PdfMetrics::BLOCK_GAP);
    }

    fn table_rows(&self, header: &[Vec<InlineRun>], rows: &[Vec<Vec<InlineRun>>], columns: usize) -> Vec<TableRow> {
        let size = PdfMetrics::TABLE_POINTS;
        let header_base = BaseStyle { size, bold: true, color: DocumentStyle::TEXT };
        let body_base = BaseStyle { size, bold: false, color: DocumentStyle::TEXT };
        let empty: Vec<InlineRun> = Vec::new();
        let build = |cells: &[Vec<InlineRun>], base: &BaseStyle, is_header: bool| TableRow {
            header: is_header,
            cells: (0..columns).map(|column| self.flow.items(cells.get(column).unwrap_or(&empty), base)).collect(),
        };
        std::iter::once(build(header, &header_base, true)).chain(rows.iter().map(|row| build(row, &body_base, false))).collect()
    }

    fn column_widths(rows: &[TableRow], columns: usize, width: f32) -> Vec<f32> {
        let padding = 2.0 * PdfMetrics::CELL_PAD_X;
        let natural: Vec<f32> = (0..columns).map(|column| Self::widest(rows, column, FlowItem::width, true) + padding).collect();
        let minimum: Vec<f32> = (0..columns).map(|column| Self::widest(rows, column, FlowItem::word_width, false) + padding).collect();
        let natural_total: f32 = natural.iter().sum();
        let minimum_total: f32 = minimum.iter().sum();
        if natural_total <= width {
            return natural.iter().map(|value| value * width / natural_total).collect();
        }
        if minimum_total >= width {
            return minimum.iter().map(|value| value * width / minimum_total).collect();
        }
        let demand: Vec<f32> = natural.iter().zip(&minimum).map(|(natural, minimum)| natural - minimum).collect();
        let demand_total: f32 = demand.iter().sum();
        let spare = width - minimum_total;
        minimum.iter().zip(&demand).map(|(minimum, demand)| minimum + spare * demand / demand_total).collect()
    }

    fn widest(rows: &[TableRow], column: usize, measure: fn(&FlowItem) -> f32, sum_line: bool) -> f32 {
        rows.iter()
            .map(|row| {
                let items = row.cells[column].iter().map(measure);
                if sum_line {
                    items.sum::<f32>()
                } else {
                    items.fold(0.0, f32::max)
                }
            })
            .fold(0.0, f32::max)
    }

    fn table_row(&mut self, row: &TableRow, widths: &[f32], x: f32) {
        let line_height = PdfMetrics::TABLE_POINTS * TABLE_LEADING;
        let cell_lines: Vec<Vec<FlowLine>> =
            row.cells.iter().zip(widths).map(|(items, width)| self.flow.lines(items, width - 2.0 * PdfMetrics::CELL_PAD_X)).collect();
        let total_lines = cell_lines.iter().map(Vec::len).max().unwrap_or(0).max(1);
        let mut offset = 0;
        while offset < total_lines {
            self.sheet.ensure(line_height + 2.0 * PdfMetrics::CELL_PAD_Y);
            let fits = (((self.sheet.room() - 2.0 * PdfMetrics::CELL_PAD_Y) / line_height).floor() as usize).max(1);
            let taken = fits.min(total_lines - offset);
            let height = taken as f32 * line_height + 2.0 * PdfMetrics::CELL_PAD_Y;
            let top = self.sheet.cursor();
            if row.header {
                self.sheet.push(Placed::Rect { x, top, width: widths.iter().sum(), height, fill: DocumentStyle::CODE_BACKGROUND });
            }
            self.draw_cells(&cell_lines, widths, x, &RowChunk { top, offset, taken, line_height });
            self.box_row(widths, x, top, height);
            self.sheet.advance(height);
            offset += taken;
            if offset < total_lines {
                self.sheet.new_page();
            }
        }
    }

    fn draw_cells(&mut self, cell_lines: &[Vec<FlowLine>], widths: &[f32], x: f32, chunk: &RowChunk) {
        let mut cell_x = x;
        for (lines, width) in cell_lines.iter().zip(widths) {
            for (position, line) in lines.iter().skip(chunk.offset).take(chunk.taken).enumerate() {
                let line_top = chunk.top + PdfMetrics::CELL_PAD_Y + position as f32 * chunk.line_height;
                self.painter.draw(&mut self.sheet, line, cell_x + PdfMetrics::CELL_PAD_X, line_top, chunk.line_height);
            }
            cell_x += width;
        }
    }

    fn box_row(&mut self, widths: &[f32], x: f32, top: f32, height: f32) {
        let right = x + widths.iter().sum::<f32>();
        let bottom = top + height;
        let mut lines = vec![(x, top, right, top), (x, bottom, right, bottom), (x, top, x, bottom)];
        let mut boundary = x;
        for width in widths {
            boundary += width;
            lines.push((boundary, top, boundary, bottom));
        }
        for (x1, y1, x2, y2) in lines {
            self.sheet.push(Placed::Stroke { x1, y1, x2, y2, width: PdfMetrics::HAIRLINE_WIDTH, color: DocumentStyle::HAIRLINE });
        }
    }
}
