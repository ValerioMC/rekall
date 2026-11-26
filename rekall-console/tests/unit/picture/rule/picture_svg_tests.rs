use image::{GenericImageView, ImageFormat, Rgba};

use super::*;

fn svg(attributes: &str, body: &str) -> Vec<u8> {
    format!(r#"<svg xmlns="http://www.w3.org/2000/svg" {attributes}>{body}</svg>"#).into_bytes()
}

fn pixel(view: &PrintablePictureView, x: u32, y: u32) -> Rgba<u8> {
    image::load_from_memory_with_format(&view.png, ImageFormat::Png).unwrap().get_pixel(x, y)
}

fn size_of(view: &PrintablePictureView) -> (u32, u32) {
    image::load_from_memory_with_format(&view.png, ImageFormat::Png).unwrap().dimensions()
}

#[test]
fn an_svg_is_drawn_at_twice_its_natural_size() {
    let drawing = drawn(&svg(r#"width="100" height="40""#, r##"<rect width="100" height="40" fill="#ff0000"/>"##)).unwrap();

    assert_eq!(size_of(&drawing), (200, 80));
    assert_eq!(drawing.natural_width, 100);
    assert_eq!(pixel(&drawing, 199, 79), Rgba([255, 0, 0, 255]));
}

#[test]
fn a_view_box_alone_gives_the_natural_size_and_the_background_stays_clear() {
    let drawing = drawn(&svg(r#"viewBox="0 0 60 30""#, "")).unwrap();

    assert_eq!(size_of(&drawing), (120, 60));
    assert_eq!(pixel(&drawing, 0, 0), Rgba([0, 0, 0, 0]));
}

#[test]
fn a_huge_svg_is_drawn_within_the_pixel_cap_and_keeps_its_natural_width() {
    let drawing = drawn(&svg(r#"width="20000" height="1000""#, "")).unwrap();

    assert_eq!(size_of(&drawing), (4096, 205));
    assert_eq!(drawing.natural_width, 20_000);
}

#[test]
fn a_file_the_svg_points_to_is_not_drawn_into_it() {
    let folder = tempfile::tempdir().unwrap();
    let secret = folder.path().join("secret.png");
    image::RgbImage::from_pixel(4, 4, image::Rgb([0, 255, 0])).save(&secret).unwrap();
    let body = format!(r#"<image href="{}" width="10" height="10"/>"#, secret.display());

    let drawing = drawn(&svg(r#"width="10" height="10""#, &body)).unwrap();

    assert_eq!(pixel(&drawing, 5, 5), Rgba([0, 0, 0, 0]));
}

#[test]
fn a_file_that_is_no_drawable_svg_is_refused() {
    for bytes in [b"<svg".to_vec(), svg(r#"width="0" height="0""#, "")] {
        assert!(drawn(&bytes).unwrap_err().is_illegal_argument());
    }
}
