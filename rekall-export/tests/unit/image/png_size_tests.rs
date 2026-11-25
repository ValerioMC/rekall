use super::*;

fn png_header(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = SIGNATURE.to_vec();
    bytes.extend_from_slice(&13u32.to_be_bytes());
    bytes.extend_from_slice(b"IHDR");
    bytes.extend_from_slice(&width.to_be_bytes());
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes
}

#[test]
fn the_size_is_read_from_the_header() {
    assert_eq!(PngSize::read(&png_header(640, 480)).unwrap(), PngSize { width: 640, height: 480 });
}

#[test]
fn bytes_that_are_not_a_png_are_refused() {
    assert!(PngSize::read(b"not a png at all, just text").is_err());
    assert!(PngSize::read(&[]).is_err());
    assert!(PngSize::read(&png_header(0, 10)).is_err());
}

#[test]
fn a_wide_image_fits_the_width_and_a_tall_one_the_height() {
    let wide = PngSize { width: 2000, height: 1000 };
    let tall = PngSize { width: 1000, height: 4000 };

    assert_eq!(wide.fit(160.0, 200.0), (160.0, 80.0));
    assert_eq!(tall.fit(160.0, 200.0), (50.0, 200.0));
}
