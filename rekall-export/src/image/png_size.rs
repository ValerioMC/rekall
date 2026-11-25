use crate::ExportError;

const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
const HEADER_LENGTH: usize = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PngSize {
    pub width: u32,
    pub height: u32,
}

impl PngSize {
    pub fn read(png: &[u8]) -> Result<Self, ExportError> {
        if png.len() < HEADER_LENGTH || png[..8] != SIGNATURE || &png[12..16] != b"IHDR" {
            return Err(ExportError::Image("the bytes are not a PNG".to_string()));
        }
        let width = u32::from_be_bytes([png[16], png[17], png[18], png[19]]);
        let height = u32::from_be_bytes([png[20], png[21], png[22], png[23]]);
        if width == 0 || height == 0 {
            return Err(ExportError::Image("the PNG has no size".to_string()));
        }
        Ok(Self { width, height })
    }

    /// The largest size with this aspect ratio that fits in the box, in the box's unit.
    pub fn fit(&self, max_width: f64, max_height: f64) -> (f64, f64) {
        let scale = (max_width / f64::from(self.width)).min(max_height / f64::from(self.height));
        (f64::from(self.width) * scale, f64::from(self.height) * scale)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/image/png_size_tests.rs"]
mod tests;
