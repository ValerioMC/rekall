use super::flow_piece::FlowPiece;

/// A word is a run of pieces with no space between them, so `**bold**,` never breaks inside.
#[derive(Debug, Clone)]
pub(super) enum FlowItem {
    Word(Vec<FlowPiece>),
    Space(FlowPiece),
    Break,
}

impl FlowItem {
    pub(super) fn width(&self) -> f32 {
        match self {
            FlowItem::Word(pieces) => pieces.iter().map(|piece| piece.width).sum(),
            FlowItem::Space(space) => space.width,
            FlowItem::Break => 0.0,
        }
    }

    pub(super) fn word_width(&self) -> f32 {
        match self {
            FlowItem::Word(_) => self.width(),
            _ => 0.0,
        }
    }
}
