use super::flow_piece::FlowPiece;

#[derive(Debug, Clone, Default)]
pub(super) struct FlowLine {
    pub(super) pieces: Vec<FlowPiece>,
    pub(super) width: f32,
    pub(super) size: f32,
}

impl FlowLine {
    pub(super) fn of(pieces: Vec<FlowPiece>) -> Self {
        let mut line = Self::default();
        line.extend(pieces);
        line
    }

    pub(super) fn is_empty(&self) -> bool {
        self.pieces.is_empty()
    }

    pub(super) fn push(&mut self, piece: FlowPiece) {
        self.width += piece.width;
        self.size = self.size.max(piece.style.size);
        self.pieces.push(piece);
    }

    pub(super) fn extend(&mut self, pieces: Vec<FlowPiece>) {
        for piece in pieces {
            self.push(piece);
        }
    }
}
