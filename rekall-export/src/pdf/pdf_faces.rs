use crate::ExportError;

use super::pdf_face::FontFace;
use super::pdf_font_kind::FontKind;

pub(super) struct PdfFaces {
    faces: Vec<FontFace>,
}

impl PdfFaces {
    pub(super) fn load() -> Result<Self, ExportError> {
        let faces = FontKind::ALL.into_iter().map(FontFace::load).collect::<Result<Vec<_>, _>>()?;
        Ok(Self { faces })
    }

    pub(super) fn face(&self, kind: FontKind) -> &FontFace {
        let position = FontKind::ALL.iter().position(|candidate| *candidate == kind).unwrap_or(0);
        &self.faces[position]
    }
}
