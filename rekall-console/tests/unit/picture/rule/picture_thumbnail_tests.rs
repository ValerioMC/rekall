use std::io::Cursor;

use image::{GenericImageView, ImageFormat, Rgb, RgbImage};

use super::*;

fn picture(width: u32, height: u32, format: ImageFormat) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    RgbImage::from_pixel(width, height, Rgb([200, 120, 40])).write_to(&mut out, format).unwrap();
    out.into_inner()
}

fn drawn(view: &PictureFileView) -> DynamicImage {
    assert_eq!(view.format, PictureFormat::Png);
    image::load_from_memory_with_format(&view.bytes, ImageFormat::Png).unwrap()
}

/// A JPEG with an EXIF segment saying it is shown turned a quarter clockwise.
fn turned_jpeg(width: u32, height: u32) -> Vec<u8> {
    let exif: &[u8] = b"Exif\0\0II*\0\x08\0\0\0\x01\0\x12\x01\x03\0\x01\0\0\0\x06\0\0\0\0\0\0\0";
    let mut jpeg = picture(width, height, ImageFormat::Jpeg);
    let mut segment = vec![0xFF, 0xE1];
    segment.extend_from_slice(&u16::try_from(exif.len() + 2).unwrap().to_be_bytes());
    segment.extend_from_slice(exif);
    jpeg.splice(2..2, segment);
    jpeg
}

#[test]
fn a_large_picture_of_every_raster_format_comes_back_as_a_png_within_the_bounds() {
    let cases = [
        (ImageFormat::Png, PictureFormat::Png),
        (ImageFormat::Jpeg, PictureFormat::Jpeg),
        (ImageFormat::Gif, PictureFormat::Gif),
        (ImageFormat::WebP, PictureFormat::Webp),
    ];
    for (written, format) in cases {
        let thumbnail = of(picture(1280, 640, written), format).unwrap();

        assert_eq!(drawn(&thumbnail).dimensions(), (320, 160), "{format:?}");
    }
}

#[test]
fn a_small_picture_is_never_enlarged() {
    let thumbnail = of(picture(40, 24, ImageFormat::Png), PictureFormat::Png).unwrap();

    assert_eq!(drawn(&thumbnail).dimensions(), (40, 24));
}

#[test]
fn a_photo_is_turned_as_its_orientation_says() {
    let thumbnail = of(turned_jpeg(40, 24), PictureFormat::Jpeg).unwrap();

    assert_eq!(drawn(&thumbnail).dimensions(), (24, 40));
}

#[test]
fn an_svg_is_returned_as_it_is() {
    let svg = br#"<svg xmlns="http://www.w3.org/2000/svg"/>"#.to_vec();

    let thumbnail = of(svg.clone(), PictureFormat::Svg).unwrap();

    assert_eq!(thumbnail, PictureFileView { bytes: svg, format: PictureFormat::Svg });
}

#[test]
fn a_file_that_only_starts_like_a_picture_is_the_callers_mistake() {
    let refused = of(b"\x89PNG\r\n\x1a\n pixels".to_vec(), PictureFormat::Png).unwrap_err();

    assert!(matches!(refused, RekallError::IllegalArgument(_)), "{refused:?}");
}
