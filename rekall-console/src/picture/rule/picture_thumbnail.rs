//! The small preview of a file a person is browsing: a PNG for the raster formats, and an SVG as it
//! is, since it draws at any size.

use std::io::Cursor;

use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader, ImageResult, Limits};
use rekall_core::picture::PictureFormat;
use rekall_kernel::{InternalFailure, RekallError, Result};

use crate::picture::PictureFileView;

pub(in crate::picture) const THUMBNAIL_SIDE: u32 = 320;
const MAX_DECODED_SIDE: u32 = 16_384;

pub(in crate::picture) fn of(bytes: Vec<u8>, format: PictureFormat) -> Result<PictureFileView> {
    if format == PictureFormat::Svg {
        return Ok(PictureFileView { bytes, format });
    }
    let image = decode(&bytes).map_err(|failure| RekallError::illegal(format!("The picture cannot be drawn: {failure}")))?;
    Ok(PictureFileView { bytes: png_of(&fitted(image))?, format: PictureFormat::Png })
}

/// Turned the way its EXIF orientation says, as a browser draws the file itself. A GIF gives its
/// first frame.
fn decode(bytes: &[u8]) -> ImageResult<DynamicImage> {
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

/// Shrunk to fit `THUMBNAIL_SIDE`, never enlarged.
fn fitted(image: DynamicImage) -> DynamicImage {
    if image.width() <= THUMBNAIL_SIDE && image.height() <= THUMBNAIL_SIDE {
        return image;
    }
    image.thumbnail(THUMBNAIL_SIDE, THUMBNAIL_SIDE)
}

fn png_of(image: &DynamicImage) -> Result<Vec<u8>> {
    let mut out = Cursor::new(Vec::new());
    image
        .write_to(&mut out, ImageFormat::Png)
        .map_err(|failure| RekallError::internal(InternalFailure::Encoding, failure))?;
    Ok(out.into_inner())
}

#[cfg(test)]
#[path = "../../../tests/unit/picture/rule/picture_thumbnail_tests.rs"]
mod tests;
