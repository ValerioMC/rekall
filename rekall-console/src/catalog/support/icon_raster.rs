//! The pixels of a project icon or a company logo: an upload of any shape becomes one square master,
//! and a render is that master resized once, straight to the size a spot on screen needs. Rounded
//! corners are not baked in; the console clips them, so they stay sharp at every size.

use std::io::Cursor;

use image::imageops::{self, FilterType};
use image::{ImageFormat, ImageReader, Limits, RgbaImage};
use rekall_kernel::{InternalFailure, RekallError, Result};

/// The master's side; a smaller upload keeps its own size rather than being enlarged.
pub const MASTER_SIDE: u32 = 512;
pub const MIN_RENDER_SIDE: u32 = 8;
pub const MAX_UPLOAD_BYTES: usize = 10 * 1024 * 1024;
const MAX_DECODED_SIDE: u32 = 8_192;

/// Runs pixel work on the blocking pool, so a large decode never stalls the async runtime.
pub async fn in_background<T: Send + 'static>(work: impl FnOnce() -> Result<T> + Send + 'static) -> Result<T> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|error| RekallError::internal(InternalFailure::Background, error))?
}

/// Decode, crop the centre to a square, shrink to `MASTER_SIDE`, and encode as PNG.
pub fn normalise(upload: &[u8]) -> Result<Vec<u8>> {
    let decoded = decode(upload)?;
    let side = decoded.width().min(decoded.height());
    let square = imageops::crop_imm(&decoded, (decoded.width() - side) / 2, (decoded.height() - side) / 2, side, side).to_image();
    let master = if side > MASTER_SIDE { resize(&square, MASTER_SIDE) } else { square };
    encode(&master)
}

/// The master at exactly `pixels` per side, never larger than the master itself.
pub fn render(master_png: &[u8], pixels: u32) -> Result<Vec<u8>> {
    let master = decode(master_png)?;
    let side = pixels.clamp(MIN_RENDER_SIDE, MASTER_SIDE);
    if side >= master.width() {
        return Ok(master_png.to_vec());
    }
    encode(&resize(&master, side))
}

fn decode(bytes: &[u8]) -> Result<RgbaImage> {
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_DECODED_SIDE);
    limits.max_image_height = Some(MAX_DECODED_SIDE);
    let mut reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format().map_err(|error| RekallError::illegal(format!("The file is not an image: {error}")))?;
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|error| RekallError::illegal(format!("The file is not a PNG, JPEG, WebP or GIF image: {error}")))?;
    Ok(image.to_rgba8())
}

fn encode(image: &RgbaImage) -> Result<Vec<u8>> {
    let mut out = Cursor::new(Vec::new());
    image
        .write_to(&mut out, ImageFormat::Png)
        .map_err(|error| RekallError::internal(InternalFailure::Encoding, error))?;
    Ok(out.into_inner())
}

/// Lanczos3 on premultiplied colour: resizing straight alpha lets the colour hidden under a
/// transparent pixel bleed a dark or light fringe into the edge of a logo.
fn resize(source: &RgbaImage, side: u32) -> RgbaImage {
    let mut premultiplied = source.clone();
    premultiplied.pixels_mut().for_each(|pixel| {
        let alpha = u32::from(pixel[3]);
        (0..3).for_each(|channel| pixel[channel] = ((u32::from(pixel[channel]) * alpha + 127) / 255) as u8);
    });
    let mut resized = imageops::resize(&premultiplied, side, side, FilterType::Lanczos3);
    resized.pixels_mut().for_each(|pixel| {
        let alpha = u32::from(pixel[3]);
        if alpha > 0 {
            (0..3).for_each(|channel| pixel[channel] = ((u32::from(pixel[channel]) * 255 + alpha / 2) / alpha).min(255) as u8);
        }
    });
    resized
}

#[cfg(test)]
#[path = "../../../tests/unit/catalog/support/icon_raster_tests.rs"]
mod tests;
