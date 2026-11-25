use printpdf::ParsedFont;

use crate::ExportError;

use super::pdf_font_kind::FontKind;

/// One parsed font: the measures the layout needs, in fractions of the font size.
pub(super) struct FontFace {
    font: ParsedFont,
    units_per_em: f32,
}

impl FontFace {
    pub(super) fn load(kind: FontKind) -> Result<Self, ExportError> {
        let font = ParsedFont::from_bytes(kind.bytes(), 0, &mut Vec::new())
            .ok_or_else(|| ExportError::Pdf(format!("the {kind:?} font could not be read")))?;
        let units_per_em = f32::from(font.pdf_font_metrics.units_per_em);
        Ok(Self { font, units_per_em })
    }

    pub(super) fn parsed(&self) -> &ParsedFont {
        &self.font
    }

    pub(super) fn glyph_of(&self, character: char) -> Option<u16> {
        self.font.lookup_glyph_index(character as u32)
    }

    pub(super) fn advance(&self, character: char) -> f32 {
        let glyph = self.glyph_of(character).unwrap_or(0);
        f32::from(self.font.get_horizontal_advance(glyph)) / self.units_per_em
    }

    pub(super) fn width(&self, text: &str, size: f32) -> f32 {
        text.chars().map(|character| self.advance(character)).sum::<f32>() * size
    }

    pub(super) fn ascent(&self) -> f32 {
        self.font.font_metrics.ascent / self.units_per_em
    }

    pub(super) fn descent(&self) -> f32 {
        -self.font.font_metrics.descent / self.units_per_em
    }
}
