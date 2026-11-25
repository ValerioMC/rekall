use crate::style::DocumentStyle;

const POINTS_PER_MM: f32 = 2.834_645_7;

/// Sizes of the PDF layout in points, measured from the page's top left corner.
pub(super) struct PdfMetrics;

impl PdfMetrics {
    pub(super) const LEADING: f32 = 1.45;
    pub(super) const CODE_LEADING: f32 = 1.4;
    pub(super) const PARAGRAPH_GAP: f32 = 6.0;
    pub(super) const TIGHT_GAP: f32 = 2.5;
    pub(super) const BLOCK_GAP: f32 = 8.0;
    pub(super) const HEADING_BEFORE: [f32; 4] = [20.0, 14.0, 12.0, 10.0];
    pub(super) const HEADING_AFTER: f32 = 6.0;
    pub(super) const KEEP_WITH_NEXT_LINES: f32 = 2.0;
    pub(super) const LIST_INDENT: f32 = 16.0;
    pub(super) const QUOTE_INDENT: f32 = 14.0;
    pub(super) const QUOTE_RULE_WIDTH: f32 = 2.0;
    pub(super) const CODE_PAD_X: f32 = 6.0;
    pub(super) const CODE_PAD_Y: f32 = 5.0;
    pub(super) const CELL_PAD_X: f32 = 5.0;
    pub(super) const CELL_PAD_Y: f32 = 3.5;
    pub(super) const TABLE_POINTS: f32 = 9.5;
    pub(super) const INLINE_CODE_SCALE: f32 = 0.9;
    pub(super) const HAIRLINE_WIDTH: f32 = 0.5;
    pub(super) const ERROR_BOX_HEIGHT: f32 = 34.0;
    pub(super) const ERROR_BORDER_WIDTH: f32 = 1.0;
    pub(super) const HEADER_BASELINE_MM: f32 = 15.0;
    pub(super) const FOOTER_BASELINE_FROM_BOTTOM_MM: f32 = 13.0;

    pub(super) fn points(millimetres: f64) -> f32 {
        millimetres as f32 * POINTS_PER_MM
    }

    pub(super) fn page_width() -> f32 {
        Self::points(DocumentStyle::PAGE_WIDTH_MM)
    }

    pub(super) fn page_height() -> f32 {
        Self::points(DocumentStyle::PAGE_HEIGHT_MM)
    }

    pub(super) fn margin() -> f32 {
        Self::points(DocumentStyle::MARGIN_MM)
    }

    pub(super) fn content_width() -> f32 {
        Self::points(DocumentStyle::TEXT_WIDTH_MM)
    }

    pub(super) fn diagram_max_height() -> f32 {
        Self::points(DocumentStyle::DIAGRAM_MAX_HEIGHT_MM)
    }
}
