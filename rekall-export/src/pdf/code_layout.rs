use crate::style::DocumentStyle;

use super::pdf_blocks::PdfBlocks;
use super::pdf_font_kind::FontKind;
use super::pdf_metrics::PdfMetrics;
use super::placed::Placed;
use super::text_style::TextStyle;

const TAB_AS_SPACES: &str = "    ";

impl<'a> PdfBlocks<'a> {
    /// A grey block of monospace lines. A line wider than the block is wrapped at characters;
    /// a block that crosses a page continues on the next one with its own grey ground.
    pub(super) fn code_block(&mut self, text: &str, x: f32, width: f32) {
        let size = DocumentStyle::CODE_POINTS as f32;
        let line_height = size * PdfMetrics::CODE_LEADING;
        let advance = self.faces.face(FontKind::Code).advance('0') * size;
        let characters_per_line = (((width - 2.0 * PdfMetrics::CODE_PAD_X) / advance).floor() as usize).max(1);
        let lines = Self::wrapped_lines(text, characters_per_line);
        let style = TextStyle { kind: FontKind::Code, size, color: DocumentStyle::TEXT, link: None };
        let mut index = 0;
        while index < lines.len() {
            self.sheet.ensure(line_height + 2.0 * PdfMetrics::CODE_PAD_Y);
            let fits = (((self.sheet.room() - 2.0 * PdfMetrics::CODE_PAD_Y) / line_height).floor() as usize).max(1);
            let taken = fits.min(lines.len() - index);
            let height = taken as f32 * line_height + 2.0 * PdfMetrics::CODE_PAD_Y;
            let top = self.sheet.cursor();
            self.sheet.push(Placed::Rect { x, top, width, height, fill: DocumentStyle::CODE_BACKGROUND });
            for (offset, line) in lines[index..index + taken].iter().enumerate() {
                let line_top = top + PdfMetrics::CODE_PAD_Y + offset as f32 * line_height;
                self.painter.text(&mut self.sheet, x + PdfMetrics::CODE_PAD_X, line_top, line_height, line, style.clone());
            }
            self.sheet.advance(height);
            index += taken;
            if index < lines.len() {
                self.sheet.new_page();
            }
        }
        self.sheet.advance(PdfMetrics::BLOCK_GAP);
    }

    fn wrapped_lines(text: &str, characters_per_line: usize) -> Vec<String> {
        text.replace('\t', TAB_AS_SPACES)
            .split('\n')
            .flat_map(|line| {
                let characters: Vec<char> = line.chars().collect();
                if characters.is_empty() {
                    vec![String::new()]
                } else {
                    characters.chunks(characters_per_line).map(|chunk| chunk.iter().collect()).collect()
                }
            })
            .collect()
    }
}
