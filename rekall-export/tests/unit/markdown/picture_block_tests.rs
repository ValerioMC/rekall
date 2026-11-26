use super::*;

const PNG_240_BY_120: &[u8] = include_bytes!("../../fixtures/diagram.png");
const NATURAL_WIDTH_MM: f64 = 63.5;

fn block(natural_width: u32) -> PictureBlock {
    PictureBlock { png: PNG_240_BY_120.to_vec(), natural_width, alignment: PictureAlignment::Left }
}

fn close(actual: (f64, f64), expected: (f64, f64)) -> bool {
    (actual.0 - expected.0).abs() < 0.001 && (actual.1 - expected.1).abs() < 0.001
}

#[test]
fn a_picture_with_room_to_spare_keeps_its_natural_width() {
    let fitted = block(240).fit(160.0, 200.0).unwrap();

    assert!(close(fitted, (NATURAL_WIDTH_MM, NATURAL_WIDTH_MM / 2.0)), "{fitted:?}");
}

#[test]
fn a_png_with_more_pixels_than_the_natural_width_is_not_drawn_wider() {
    let fitted = block(120).fit(160.0, 200.0).unwrap();

    assert!(close(fitted, (NATURAL_WIDTH_MM / 2.0, NATURAL_WIDTH_MM / 4.0)), "{fitted:?}");
}

#[test]
fn a_picture_wider_or_taller_than_the_box_is_shrunk_into_it() {
    assert!(close(block(2000).fit(160.0, 200.0).unwrap(), (160.0, 80.0)));
    assert!(close(block(2000).fit(160.0, 40.0).unwrap(), (80.0, 40.0)));
}

#[test]
fn bytes_that_are_not_a_png_are_refused() {
    let broken = PictureBlock { png: b"not a png".to_vec(), ..block(240) };

    assert!(matches!(broken.fit(160.0, 200.0), Err(ExportError::Image(_))));
}
