use std::collections::{BTreeMap, BTreeSet};

use printpdf::{
    Actions, BorderArray, Color, Line, LinePoint, LinkAnnotation, Mm, Op, PaintMode, ParsedFont, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions, Point,
    Polygon, PolygonRing, Pt, RawImage, Rect, Rgb as PdfRgb, TextItem, WindingOrder, XObjectId, XObjectTransform,
};

use crate::style::{DocumentStyle, Rgb};
use crate::ExportError;

use super::pdf_faces::PdfFaces;
use super::pdf_font_kind::FontKind;
use super::pdf_metrics::PdfMetrics;
use super::placed::Placed;
use super::text_style::TextStyle;

const POINTS_PER_INCH: f32 = 72.0;
const ELLIPSIS: char = '…';

struct Picture {
    id: XObjectId,
    pixel_width: f32,
}

/// Writes the laid-out pages with printpdf. Only the glyphs a font draws are embedded, because
/// printpdf embeds a font whole otherwise, and the finished file is deflated.
pub(super) struct PdfEmitter<'a> {
    faces: &'a PdfFaces,
    title: &'a str,
}

impl<'a> PdfEmitter<'a> {
    pub(super) fn new(faces: &'a PdfFaces, title: &'a str) -> Self {
        Self { faces, title }
    }

    pub(super) fn emit(&self, mut pages: Vec<Vec<Placed>>, pictures: &[&[u8]]) -> Result<Vec<u8>, ExportError> {
        self.add_page_furniture(&mut pages);
        let mut document = PdfDocument::new(self.title);
        let fonts = self.register_fonts(&mut document, &pages)?;
        let images = Self::register_pictures(&mut document, pictures)?;
        let pdf_pages: Vec<PdfPage> = pages
            .iter()
            .map(|page| PdfPage::new(Mm(DocumentStyle::PAGE_WIDTH_MM as f32), Mm(DocumentStyle::PAGE_HEIGHT_MM as f32), Self::operations(page, &fonts, &images)))
            .collect();
        let mut warnings = Vec::new();
        let bytes = document.with_pages(pdf_pages).save(&Self::save_options(), &mut warnings);
        if let Some(failure) = warnings.iter().find(|warning| format!("{:?}", warning.severity) == "Error") {
            return Err(ExportError::Pdf(failure.msg.clone()));
        }
        Self::deflate(&bytes)
    }

    /// printpdf otherwise shrinks every image past 2 MB and recompresses it as JPEG, which blurs
    /// the text of a screenshot. `deflate` compresses the pixels without loss.
    fn save_options() -> PdfSaveOptions {
        PdfSaveOptions { image_optimization: None, ..PdfSaveOptions::default() }
    }

    fn add_page_furniture(&self, pages: &mut [Vec<Placed>]) {
        let total = pages.len();
        let size = DocumentStyle::FURNITURE_POINTS as f32;
        let style = TextStyle { kind: FontKind::Body, size, color: DocumentStyle::MUTED, link: None };
        let face = self.faces.face(FontKind::Body);
        let header = self.fitted_title(size);
        let footer_baseline = PdfMetrics::page_height() - PdfMetrics::points(f64::from(PdfMetrics::FOOTER_BASELINE_FROM_BOTTOM_MM));
        for (index, page) in pages.iter_mut().enumerate() {
            page.push(Placed::Text { x: PdfMetrics::margin(), baseline: PdfMetrics::points(f64::from(PdfMetrics::HEADER_BASELINE_MM)), text: header.clone(), style: style.clone() });
            let numbering = format!("{} / {}", index + 1, total);
            let x = (PdfMetrics::page_width() - face.width(&numbering, size)) / 2.0;
            page.push(Placed::Text { x, baseline: footer_baseline, text: numbering, style: style.clone() });
        }
    }

    fn fitted_title(&self, size: f32) -> String {
        let face = self.faces.face(FontKind::Body);
        let mut title: String = self.title.chars().filter(|character| !character.is_control()).collect();
        if face.width(&title, size) <= PdfMetrics::content_width() {
            return title;
        }
        while !title.is_empty() && face.width(&format!("{title}{ELLIPSIS}"), size) > PdfMetrics::content_width() {
            title.pop();
        }
        format!("{title}{ELLIPSIS}")
    }

    fn register_fonts(&self, document: &mut PdfDocument, pages: &[Vec<Placed>]) -> Result<BTreeMap<FontKind, printpdf::FontId>, ExportError> {
        let mut used: BTreeMap<FontKind, BTreeSet<char>> = BTreeMap::new();
        for placed in pages.iter().flatten() {
            if let Placed::Text { text, style, .. } = placed {
                used.entry(style.kind).or_default().extend(text.chars());
            }
        }
        let mut fonts = BTreeMap::new();
        for (kind, characters) in used {
            let subset = self.subset_font(kind, &characters)?;
            fonts.insert(kind, document.add_font(&subset));
        }
        Ok(fonts)
    }

    fn subset_font(&self, kind: FontKind, characters: &BTreeSet<char>) -> Result<ParsedFont, ExportError> {
        let face = self.faces.face(kind);
        let glyphs: BTreeMap<u16, char> = characters.iter().filter_map(|character| face.glyph_of(*character).map(|glyph| (glyph, *character))).collect();
        let subset = printpdf::font::subset_font(face.parsed(), &glyphs).map_err(|failure| ExportError::Pdf(format!("a font could not be subset: {failure}")))?;
        ParsedFont::from_bytes(&subset.bytes, 0, &mut Vec::new()).ok_or_else(|| ExportError::Pdf("a subset font could not be read".to_string()))
    }

    fn register_pictures(document: &mut PdfDocument, pictures: &[&[u8]]) -> Result<Vec<Picture>, ExportError> {
        pictures
            .iter()
            .map(|png| {
                let raw = RawImage::decode_from_bytes(png, &mut Vec::new()).map_err(|failure| ExportError::Image(format!("an image could not be decoded: {failure}")))?;
                Ok(Picture { pixel_width: raw.width as f32, id: document.add_image(&raw) })
            })
            .collect()
    }

    fn operations(page: &[Placed], fonts: &BTreeMap<FontKind, printpdf::FontId>, pictures: &[Picture]) -> Vec<Op> {
        let page_height = PdfMetrics::page_height();
        let mut operations = Vec::new();
        for placed in page {
            match placed {
                Placed::Text { x, baseline, text, style } => {
                    let Some(font) = fonts.get(&style.kind) else { continue };
                    operations.extend([
                        Op::StartTextSection,
                        Op::SetFont { font: PdfFontHandle::External(font.clone()), size: Pt(style.size) },
                        Op::SetFillColor { col: Self::color(style.color) },
                        Op::SetTextCursor { pos: Point { x: Pt(*x), y: Pt(page_height - baseline) } },
                        Op::ShowText { items: vec![TextItem::Text(text.clone())] },
                        Op::EndTextSection,
                    ]);
                }
                Placed::Rect { x, top, width, height, fill } => operations.extend([
                    Op::SetFillColor { col: Self::color(*fill) },
                    Op::DrawPolygon { polygon: Self::filled_rectangle(*x, page_height - top - height, *width, *height) },
                ]),
                Placed::Stroke { x1, y1, x2, y2, width, color } => operations.extend([
                    Op::SetOutlineColor { col: Self::color(*color) },
                    Op::SetOutlineThickness { pt: Pt(*width) },
                    Op::DrawLine {
                        line: Line {
                            points: vec![
                                LinePoint { p: Point { x: Pt(*x1), y: Pt(page_height - y1) }, bezier: false },
                                LinePoint { p: Point { x: Pt(*x2), y: Pt(page_height - y2) }, bezier: false },
                            ],
                            is_closed: false,
                        },
                    },
                ]),
                Placed::Picture { picture, x, top, width, height } => {
                    let Some(image) = pictures.get(*picture) else { continue };
                    operations.push(Op::UseXobject {
                        id: image.id.clone(),
                        transform: XObjectTransform {
                            translate_x: Some(Pt(*x)),
                            translate_y: Some(Pt(page_height - top - height)),
                            rotate: None,
                            scale_x: None,
                            scale_y: None,
                            dpi: Some(image.pixel_width * POINTS_PER_INCH / width),
                        },
                    });
                }
                Placed::Link { x, top, width, height, url } => operations.push(Op::LinkAnnotation {
                    link: LinkAnnotation::new(
                        Rect { x: Pt(*x), y: Pt(page_height - top - height), width: Pt(*width), height: Pt(*height), mode: None, winding_order: None },
                        Actions::Uri(url.clone()),
                        Some(BorderArray::Solid([0.0, 0.0, 0.0])),
                        None,
                        None,
                    ),
                }),
            }
        }
        operations
    }

    fn filled_rectangle(x: f32, bottom: f32, width: f32, height: f32) -> Polygon {
        let corners = [(x, bottom), (x + width, bottom), (x + width, bottom + height), (x, bottom + height)];
        let points = corners.into_iter().map(|(corner_x, corner_y)| LinePoint { p: Point { x: Pt(corner_x), y: Pt(corner_y) }, bezier: false }).collect();
        Polygon { rings: vec![PolygonRing { points }], mode: PaintMode::Fill, winding_order: WindingOrder::NonZero }
    }

    fn color(color: Rgb) -> Color {
        Color::Rgb(PdfRgb::new(f32::from(color.0) / 255.0, f32::from(color.1) / 255.0, f32::from(color.2) / 255.0, None))
    }

    fn deflate(bytes: &[u8]) -> Result<Vec<u8>, ExportError> {
        let mut document = lopdf::Document::load_mem(bytes).map_err(|failure| ExportError::Pdf(format!("the PDF could not be read back: {failure}")))?;
        document.compress();
        let mut compressed = Vec::new();
        document.save_to(&mut compressed).map_err(|failure| ExportError::Pdf(format!("the PDF could not be compressed: {failure}")))?;
        Ok(compressed)
    }
}
