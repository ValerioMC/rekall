//! The small preview of a file a person is browsing: a PNG for the raster formats, and an SVG as it
//! is, since it draws at any size.

use image::DynamicImage;
use rekall_core::picture::PictureFormat;
use rekall_kernel::{RekallError, Result};

use super::picture_raster;
use crate::picture::PictureFileView;

pub(in crate::picture) const THUMBNAIL_SIDE: u32 = 320;

pub(in crate::picture) fn of(bytes: Vec<u8>, format: PictureFormat) -> Result<PictureFileView> {
    if format == PictureFormat::Svg {
        return Ok(PictureFileView { bytes, format });
    }
    let image = picture_raster::decode(&bytes).map_err(|failure| RekallError::illegal(format!("The picture cannot be drawn: {failure}")))?;
    Ok(PictureFileView { bytes: picture_raster::png_of(&fitted(image))?, format: PictureFormat::Png })
}

/// Shrunk to fit `THUMBNAIL_SIDE`, never enlarged.
fn fitted(image: DynamicImage) -> DynamicImage {
    if image.width() <= THUMBNAIL_SIDE && image.height() <= THUMBNAIL_SIDE {
        return image;
    }
    image.thumbnail(THUMBNAIL_SIDE, THUMBNAIL_SIDE)
}

#[cfg(test)]
#[path = "../../../tests/unit/picture/rule/picture_thumbnail_tests.rs"]
mod tests;
