use crate::style::DocumentStyle;

use super::flow_line::FlowLine;
use super::flow_piece::FlowPiece;
use super::pdf_faces::PdfFaces;
use super::pdf_font_kind::FontKind;
use super::placed::Placed;
use super::sheet::Sheet;
use super::text_style::TextStyle;

const UNDERLINE_OFFSET: f32 = 0.12;
const UNDERLINE_WIDTH: f32 = 0.5;

/// Puts a flowed line on the page, centred vertically in its line box. Neighbouring pieces with
/// one style become one text so the file carries few text operations.
pub(super) struct LinePainter<'a> {
    faces: &'a PdfFaces,
}

struct Run {
    x: f32,
    width: f32,
    text: String,
    style: TextStyle,
}

impl<'a> LinePainter<'a> {
    pub(super) fn new(faces: &'a PdfFaces) -> Self {
        Self { faces }
    }

    pub(super) fn baseline(&self, kind: FontKind, size: f32, top: f32, line_height: f32) -> f32 {
        let face = self.faces.face(kind);
        let content = (face.ascent() + face.descent()) * size;
        top + (line_height - content) / 2.0 + face.ascent() * size
    }

    pub(super) fn draw(&self, sheet: &mut Sheet, line: &FlowLine, x: f32, top: f32, line_height: f32) {
        let mut pending: Option<Run> = None;
        let mut cursor = x;
        for piece in &line.pieces {
            pending = match pending.take() {
                Some(run) if run.style == piece.style => Some(Self::extended(run, piece)),
                Some(run) => {
                    self.place(sheet, run, top, line_height);
                    Some(Self::started(cursor, piece))
                }
                None => Some(Self::started(cursor, piece)),
            };
            cursor += piece.width;
        }
        if let Some(run) = pending {
            self.place(sheet, run, top, line_height);
        }
    }

    pub(super) fn text(&self, sheet: &mut Sheet, x: f32, top: f32, line_height: f32, text: &str, style: TextStyle) {
        let baseline = self.baseline(style.kind, style.size, top, line_height);
        sheet.push(Placed::Text { x, baseline, text: text.to_string(), style });
    }

    fn started(x: f32, piece: &FlowPiece) -> Run {
        Run { x, width: piece.width, text: piece.text.clone(), style: piece.style.clone() }
    }

    fn extended(mut run: Run, piece: &FlowPiece) -> Run {
        run.text.push_str(&piece.text);
        run.width += piece.width;
        run
    }

    fn place(&self, sheet: &mut Sheet, run: Run, top: f32, line_height: f32) {
        let baseline = self.baseline(run.style.kind, run.style.size, top, line_height);
        if let Some(url) = run.style.link.clone() {
            let underline = baseline + run.style.size * UNDERLINE_OFFSET;
            sheet.push(Placed::Link { x: run.x, top, width: run.width, height: line_height, url });
            sheet.push(Placed::Stroke { x1: run.x, y1: underline, x2: run.x + run.width, y2: underline, width: UNDERLINE_WIDTH, color: DocumentStyle::LINK });
        }
        sheet.push(Placed::Text { x: run.x, baseline, text: run.text, style: run.style });
    }
}
