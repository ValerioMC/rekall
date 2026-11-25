use super::text_style::TextStyle;

#[derive(Debug, Clone)]
pub(super) struct FlowPiece {
    pub(super) text: String,
    pub(super) style: TextStyle,
    pub(super) width: f32,
}
