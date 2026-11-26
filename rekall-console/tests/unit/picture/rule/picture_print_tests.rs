use std::io::Cursor;

use image::{GenericImageView, ImageBuffer, ImageFormat, Rgb, RgbImage, Rgba};

use super::*;

fn picture(width: u32, height: u32, format: ImageFormat) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    RgbImage::from_pixel(width, height, Rgb([200, 120, 40])).write_to(&mut out, format).unwrap();
    out.into_inner()
}

fn decoded(view: &PrintablePictureView) -> DynamicImage {
    image::load_from_memory_with_format(&view.png, ImageFormat::Png).unwrap()
}

#[test]
fn a_png_is_printed_as_it_is() {
    let png = picture(40, 24, ImageFormat::Png);

    let printable = of(png.clone(), PictureFormat::Png).unwrap();

    assert_eq!(printable, PrintablePictureView { png, natural_width: 40 });
}

#[test]
fn the_other_raster_formats_become_a_png_of_their_own_size() {
    let cases = [(ImageFormat::Jpeg, PictureFormat::Jpeg), (ImageFormat::Gif, PictureFormat::Gif), (ImageFormat::WebP, PictureFormat::Webp)];
    for (written, format) in cases {
        let printable = of(picture(400, 240, written), format).unwrap();

        assert_eq!(decoded(&printable).dimensions(), (400, 240), "{format:?}");
        assert_eq!(printable.natural_width, 400, "{format:?}");
    }
}

#[test]
fn a_png_deeper_than_eight_bits_is_rewritten_in_eight() {
    let mut deep = Cursor::new(Vec::new());
    ImageBuffer::from_pixel(8, 4, Rgba([60_000u16, 100, 100, 30_000])).write_to(&mut deep, ImageFormat::Png).unwrap();

    let printable = of(deep.into_inner(), PictureFormat::Png).unwrap();

    assert!(matches!(decoded(&printable), DynamicImage::ImageRgba8(_)));
}

#[test]
fn a_damaged_picture_is_refused() {
    let mut png = picture(40, 24, ImageFormat::Png);
    png.truncate(40);

    assert!(of(png, PictureFormat::Png).unwrap_err().is_illegal_argument());
}
