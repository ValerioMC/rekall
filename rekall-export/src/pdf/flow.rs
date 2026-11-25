use crate::markdown::InlineRun;
use crate::style::DocumentStyle;

use super::base_style::BaseStyle;
use super::flow_item::FlowItem;
use super::flow_line::FlowLine;
use super::flow_piece::FlowPiece;
use super::pdf_faces::PdfFaces;
use super::pdf_font_kind::FontKind;
use super::pdf_metrics::PdfMetrics;
use super::text_style::TextStyle;

const FIT_TOLERANCE: f32 = 0.01;
const MIN_SHARE_TO_CONTINUE_LINE: f32 = 0.25;

/// Breaks styled text into lines of a given width. A word wider than a whole line is cut at
/// characters, so nothing ever runs off the page or is lost.
pub(super) struct Flow<'a> {
    faces: &'a PdfFaces,
}

impl<'a> Flow<'a> {
    pub(super) fn new(faces: &'a PdfFaces) -> Self {
        Self { faces }
    }

    pub(super) fn items(&self, inlines: &[InlineRun], base: &BaseStyle) -> Vec<FlowItem> {
        let mut items = Vec::new();
        let mut word: Vec<FlowPiece> = Vec::new();
        for run in inlines {
            let style = Self::style_of(run, base);
            let mut text = String::new();
            for character in run.text.chars() {
                if character == '\n' {
                    self.push_text(&mut word, &mut text, &style);
                    Self::close_word(&mut items, &mut word);
                    items.push(FlowItem::Break);
                } else if character.is_whitespace() {
                    self.push_text(&mut word, &mut text, &style);
                    Self::close_word(&mut items, &mut word);
                    if !matches!(items.last(), Some(FlowItem::Space(_))) {
                        items.push(FlowItem::Space(self.piece(" ".to_string(), style.clone())));
                    }
                } else {
                    text.push(character);
                }
            }
            self.push_text(&mut word, &mut text, &style);
        }
        Self::close_word(&mut items, &mut word);
        items
    }

    pub(super) fn lines(&self, items: &[FlowItem], max_width: f32) -> Vec<FlowLine> {
        let mut lines = Vec::new();
        let mut line = FlowLine::default();
        let mut pending_space: Option<FlowPiece> = None;
        for item in items {
            match item {
                FlowItem::Break => {
                    lines.push(std::mem::take(&mut line));
                    pending_space = None;
                }
                FlowItem::Space(space) => {
                    if !line.is_empty() {
                        pending_space = Some(space.clone());
                    }
                }
                FlowItem::Word(word) => self.place_word(word, max_width, &mut lines, &mut line, &mut pending_space),
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
        lines
    }

    fn place_word(&self, word: &[FlowPiece], max_width: f32, lines: &mut Vec<FlowLine>, line: &mut FlowLine, pending_space: &mut Option<FlowPiece>) {
        let word_width: f32 = word.iter().map(|piece| piece.width).sum();
        let space_width = pending_space.as_ref().map_or(0.0, |space| space.width);
        if line.width + space_width + word_width <= max_width + FIT_TOLERANCE {
            if let Some(space) = pending_space.take() {
                line.push(space);
            }
            line.extend(word.to_vec());
        } else if word_width <= max_width + FIT_TOLERANCE {
            lines.push(std::mem::take(line));
            *pending_space = None;
            line.extend(word.to_vec());
        } else {
            let remaining = max_width - line.width - space_width;
            let first_width = if !line.is_empty() && remaining >= max_width * MIN_SHARE_TO_CONTINUE_LINE {
                if let Some(space) = pending_space.take() {
                    line.push(space);
                }
                remaining
            } else {
                if !line.is_empty() {
                    lines.push(std::mem::take(line));
                }
                *pending_space = None;
                max_width
            };
            let mut chunks = self.cut(word, first_width, max_width);
            let last = chunks.pop();
            if let Some(first) = chunks.first_mut().filter(|_| !line.is_empty()) {
                line.extend(std::mem::take(first));
                lines.push(std::mem::take(line));
                chunks.remove(0);
            }
            lines.extend(chunks.into_iter().map(FlowLine::of));
            if let Some(last) = last {
                line.extend(last);
            }
        }
    }

    fn cut(&self, word: &[FlowPiece], first_width: f32, max_width: f32) -> Vec<Vec<FlowPiece>> {
        let mut limit = first_width;
        let mut chunks: Vec<Vec<FlowPiece>> = Vec::new();
        let mut current: Vec<FlowPiece> = Vec::new();
        let mut current_width = 0.0;
        for piece in word {
            let face = self.faces.face(piece.style.kind);
            let mut text = String::new();
            let mut text_width = 0.0;
            for character in piece.text.chars() {
                let advance = face.advance(character) * piece.style.size;
                if current_width + text_width + advance > limit && current_width + text_width > 0.0 {
                    if !text.is_empty() {
                        current.push(FlowPiece { text: std::mem::take(&mut text), style: piece.style.clone(), width: text_width });
                    }
                    chunks.push(std::mem::take(&mut current));
                    limit = max_width;
                    current_width = 0.0;
                    text_width = 0.0;
                }
                text.push(character);
                text_width += advance;
            }
            if !text.is_empty() {
                current.push(FlowPiece { text, style: piece.style.clone(), width: text_width });
                current_width += text_width;
            }
        }
        if !current.is_empty() {
            chunks.push(current);
        }
        chunks
    }

    fn style_of(run: &InlineRun, base: &BaseStyle) -> TextStyle {
        let kind = FontKind::of(base.bold || run.bold, run.italic, run.code);
        let size = if run.code { base.size * PdfMetrics::INLINE_CODE_SCALE } else { base.size };
        let color = if run.link.is_some() { DocumentStyle::LINK } else { base.color };
        TextStyle { kind, size, color, link: run.link.clone() }
    }

    fn piece(&self, text: String, style: TextStyle) -> FlowPiece {
        let width = self.faces.face(style.kind).width(&text, style.size);
        FlowPiece { text, style, width }
    }

    fn push_text(&self, word: &mut Vec<FlowPiece>, text: &mut String, style: &TextStyle) {
        if !text.is_empty() {
            word.push(self.piece(std::mem::take(text), style.clone()));
        }
    }

    fn close_word(items: &mut Vec<FlowItem>, word: &mut Vec<FlowPiece>) {
        if !word.is_empty() {
            items.push(FlowItem::Word(std::mem::take(word)));
        }
    }
}
