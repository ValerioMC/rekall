//! What a file has to be to count as a picture, judged by its content and never by its name.

use image::ImageFormat;
use rekall_core::picture::PictureFormat;
use rekall_kernel::{RekallError, Result};
use roxmltree::{Document, ParsingOptions};

pub(in crate::picture) const MAX_PICTURE_BYTES: u64 = 10 * 1024 * 1024;

pub(in crate::picture) fn format_of(bytes: &[u8]) -> Result<PictureFormat> {
    match image::guess_format(bytes) {
        Ok(ImageFormat::Png) => Ok(PictureFormat::Png),
        Ok(ImageFormat::Jpeg) => Ok(PictureFormat::Jpeg),
        Ok(ImageFormat::Gif) => Ok(PictureFormat::Gif),
        Ok(ImageFormat::WebP) => Ok(PictureFormat::Webp),
        _ if is_svg(bytes) => Ok(PictureFormat::Svg),
        _ => Err(RekallError::illegal("The file is not a PNG, JPEG, GIF, WebP or SVG picture.")),
    }
}

/// Drawing tools often declare a DTD, so one is allowed; the parser bounds its entity expansion.
fn is_svg(bytes: &[u8]) -> bool {
    let options = ParsingOptions { allow_dtd: true, ..ParsingOptions::default() };
    std::str::from_utf8(bytes)
        .ok()
        .and_then(|text| Document::parse_with_options(text, options).ok())
        .is_some_and(|document| document.root_element().tag_name().name() == "svg")
}

#[cfg(test)]
#[path = "../../../tests/unit/picture/rule/picture_content_tests.rs"]
mod tests;
