//! The PNG an exported document prints a picture from.

use image::DynamicImage;
use rekall_core::picture::PictureFormat;
use rekall_export::PngSize;
use rekall_kernel::{RekallError, Result};

use super::{picture_raster, picture_svg};
use crate::picture::PrintablePictureView;

/// A PNG goes in as it is and the other raster formats are converted; an SVG is drawn.
pub(in crate::picture) fn of(bytes: Vec<u8>, format: PictureFormat) -> Result<PrintablePictureView> {
    if format == PictureFormat::Svg {
        return picture_svg::drawn(&bytes);
    }
    let image = picture_raster::decode(&bytes).map_err(|failure| unreadable(&failure))?;
    let png = if format == PictureFormat::Png && is_eight_bit(&image) { bytes } else { picture_raster::png_of(&eight_bit(image))? };
    let natural_width = PngSize::read(&png).map_err(|failure| unreadable(&failure))?.width;
    Ok(PrintablePictureView { png, natural_width })
}

/// The PDF writer draws 8 bits a channel: a deeper picture comes out blank or broken there.
fn is_eight_bit(image: &DynamicImage) -> bool {
    matches!(image, DynamicImage::ImageLuma8(_) | DynamicImage::ImageLumaA8(_) | DynamicImage::ImageRgb8(_) | DynamicImage::ImageRgba8(_))
}

fn eight_bit(image: DynamicImage) -> DynamicImage {
    if is_eight_bit(&image) {
        return image;
    }
    DynamicImage::ImageRgba8(image.to_rgba8())
}

fn unreadable(failure: &dyn std::fmt::Display) -> RekallError {
    RekallError::illegal(format!("The picture cannot be drawn: {failure}"))
}

#[cfg(test)]
#[path = "../../../tests/unit/picture/rule/picture_print_tests.rs"]
mod tests;
