use crate::style::Rgb;

/// What plain text of a block looks like; a run's own bold, italic, code and link come on top.
pub(super) struct BaseStyle {
    pub(super) size: f32,
    pub(super) bold: bool,
    pub(super) color: Rgb,
}
