use crate::style::Rgb;

use super::pdf_font_kind::FontKind;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct TextStyle {
    pub(super) kind: FontKind,
    pub(super) size: f32,
    pub(super) color: Rgb,
    pub(super) link: Option<String>,
}
