//! A raster picture decoded within bounds, and written back as a PNG.

use std::io::Cursor;

use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader, ImageResult, Limits};
use rekall_kernel::{InternalFailure, RekallError, Result};

const MAX_DECODED_SIDE: u32 = 16_384;

/// Turned the way its EXIF orientation says, as a browser draws the file itself. A GIF gives its
/// first frame.
pub(in crate::picture) fn decode(bytes: &[u8]) -> ImageResult<DynamicImage> {
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_DECODED_SIDE);
    limits.max_image_height = Some(MAX_DECODED_SIDE);
    let mut reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
    reader.limits(limits.clone());
    let mut decoder = reader.into_decoder()?;
    limits.reserve(decoder.total_bytes())?;
    let orientation = decoder.orientation()?;
    let mut image = DynamicImage::from_decoder(decoder)?;
    image.apply_orientation(orientation);
    Ok(image)
}

pub(in crate::picture) fn png_of(image: &DynamicImage) -> Result<Vec<u8>> {
    let mut out = Cursor::new(Vec::new());
    image
        .write_to(&mut out, ImageFormat::Png)
        .map_err(|failure| RekallError::internal(InternalFailure::Encoding, failure))?;
    Ok(out.into_inner())
}
