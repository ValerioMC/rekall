use super::Rgb;

/// Page, type and colours of an exported document. Millimetres for the page, points for type.
pub struct DocumentStyle;

impl DocumentStyle {
    pub const PAGE_WIDTH_MM: f64 = 210.0;
    pub const PAGE_HEIGHT_MM: f64 = 297.0;
    pub const MARGIN_MM: f64 = 25.0;

    pub const BODY_POINTS: f64 = 10.5;
    pub const CODE_POINTS: f64 = 9.0;
    pub const HEADING_POINTS: [f64; 4] = [22.0, 16.0, 13.0, 11.5];
    pub const MAX_HEADING_LEVEL: u8 = 4;

    pub const DOCX_BODY_FONT: &'static str = "Calibri";
    pub const DOCX_CODE_FONT: &'static str = "Consolas";
    pub const PDF_BODY_FONT: &'static str = "Fira Sans";
    pub const PDF_CODE_FONT: &'static str = "Fira Code";

    pub const TEXT: Rgb = Rgb(0x1F, 0x23, 0x28);
    pub const LINK: Rgb = Rgb(0x0B, 0x5C, 0xAD);
    pub const CODE_BACKGROUND: Rgb = Rgb(0xF3, 0xF4, 0xF6);
    pub const HAIRLINE: Rgb = Rgb(0xD0, 0xD7, 0xDE);
    pub const QUOTE_RULE: Rgb = Rgb(0x9A, 0xA4, 0xB0);
    pub const ERROR_TEXT_COLOUR: Rgb = Rgb(0x9B, 0x1C, 0x1C);
    pub const ERROR_BACKGROUND: Rgb = Rgb(0xFD, 0xEC, 0xEA);
    pub const ERROR_BORDER: Rgb = Rgb(0xC0, 0x39, 0x2B);

    /// Printed in place of a diagram that could not be drawn; both writers use these words.
    pub const DIAGRAM_ERROR_TEXT: &'static str = "Error exported diagram";
}
