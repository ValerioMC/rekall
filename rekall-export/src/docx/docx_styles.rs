use docx_rs::{
    AbstractNumbering, AlignmentType, Docx, Level, LevelJc, LevelText, LineSpacing, NumberFormat, PageMargin, RunFonts, SpecialIndentType, Start, Style,
    StyleType,
};

use crate::style::DocumentStyle;

use super::docx_measure::DocxMeasure;

pub(super) const BULLET_ABSTRACT_ID: usize = 1;
pub(super) const DECIMAL_ABSTRACT_ID: usize = 2;
pub(super) const LIST_LEVELS: usize = 4;
const BULLETS: [&str; LIST_LEVELS] = ["•", "–", "▪", "–"];
const HANGING_MM: f64 = 5.0;
const LIST_STEP_MM: f64 = 10.0;

pub(super) struct DocxStyles;

impl DocxStyles {
    pub(super) fn apply(docx: Docx) -> Docx {
        let body = DocumentStyle::DOCX_BODY_FONT;
        let page_margin = DocxMeasure::twips(DocumentStyle::MARGIN_MM);
        let docx = docx
            .page_size(DocxMeasure::twips(DocumentStyle::PAGE_WIDTH_MM) as u32, DocxMeasure::twips(DocumentStyle::PAGE_HEIGHT_MM) as u32)
            .page_margin(PageMargin::new().top(page_margin).bottom(page_margin).left(page_margin).right(page_margin).header(709).footer(709))
            .default_fonts(RunFonts::new().ascii(body).hi_ansi(body).east_asia(body).cs(body))
            .default_size(DocxMeasure::half_points(DocumentStyle::BODY_POINTS))
            .default_line_spacing(LineSpacing::new().after(120).line(264))
            .add_abstract_numbering(Self::numbering(BULLET_ABSTRACT_ID, false))
            .add_abstract_numbering(Self::numbering(DECIMAL_ABSTRACT_ID, true));
        (0..DocumentStyle::MAX_HEADING_LEVEL).fold(docx, |docx, index| docx.add_style(Self::heading_style(index)))
    }

    pub(super) fn list_indent(depth: usize) -> i32 {
        DocxMeasure::twips(Self::list_indent_mm(depth))
    }

    pub(super) fn list_indent_mm(depth: usize) -> f64 {
        LIST_STEP_MM * (depth + 1) as f64
    }

    fn heading_style(index: u8) -> Style {
        let level = usize::from(index) + 1;
        Style::new(format!("Heading{level}"), StyleType::Paragraph)
            .name(format!("heading {level}"))
            .based_on("Normal")
            .next("Normal")
            .size(DocxMeasure::half_points(DocumentStyle::HEADING_POINTS[usize::from(index)]))
            .bold()
            .color(DocumentStyle::TEXT.hex())
            .line_spacing(LineSpacing::new().before(if level == 1 { 360 } else { 240 }).after(120))
            .outline_lvl(level - 1)
    }

    fn numbering(id: usize, decimal: bool) -> AbstractNumbering {
        (0..LIST_LEVELS).fold(AbstractNumbering::new(id), |numbering, depth| numbering.add_level(Self::level(depth, decimal)))
    }

    fn level(depth: usize, decimal: bool) -> Level {
        let (format, text) = if decimal { ("decimal", format!("%{}.", depth + 1)) } else { ("bullet", BULLETS[depth].to_string()) };
        Level::new(depth, Start::new(1), NumberFormat::new(format), LevelText::new(text), LevelJc::new(AlignmentType::Left.to_string()))
            .indent(Some(Self::list_indent(depth)), Some(SpecialIndentType::Hanging(DocxMeasure::twips(HANGING_MM))), None, None)
    }
}
