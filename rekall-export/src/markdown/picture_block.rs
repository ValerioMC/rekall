use crate::image::PngSize;
use crate::style::DocumentStyle;
use crate::ExportError;

use super::PictureAlignment;

/// A picture as a writer places it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PictureBlock {
    pub png: Vec<u8>,
    pub natural_width: u32,
    pub alignment: PictureAlignment,
}

impl PictureBlock {
    /// Its width and height in millimetres: shrunk to fit the box, never past its natural width.
    pub fn fit(&self, max_width_mm: f64, max_height_mm: f64) -> Result<(f64, f64), ExportError> {
        let natural_width_mm = f64::from(self.natural_width) * DocumentStyle::MILLIMETRES_PER_PIXEL;
        Ok(PngSize::read(&self.png)?.fit(natural_width_mm.min(max_width_mm), max_height_mm))
    }
}

#[cfg(test)]
#[path = "../../tests/unit/markdown/picture_block_tests.rs"]
mod tests;
