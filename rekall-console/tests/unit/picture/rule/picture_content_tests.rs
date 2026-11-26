use super::*;

#[test]
fn a_raster_picture_is_recognised_by_its_first_bytes() {
    let cases: [(&[u8], PictureFormat); 4] = [
        (b"\x89PNG\r\n\x1a\n rest", PictureFormat::Png),
        (b"\xff\xd8\xff\xe0 rest", PictureFormat::Jpeg),
        (b"GIF89a rest", PictureFormat::Gif),
        (b"RIFF\0\0\0\0WEBPVP8 rest", PictureFormat::Webp),
    ];

    for (bytes, expected) in cases {
        assert_eq!(format_of(bytes).unwrap(), expected);
    }
}

#[test]
fn xml_rooted_at_svg_is_a_picture_with_or_without_a_declaration_and_a_doctype() {
    let plain = br#"<svg xmlns="http://www.w3.org/2000/svg"/>"#;
    let declared = b"\xef\xbb\xbf<?xml version=\"1.0\"?>\n<!-- drawn -->\n<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1//EN\" \"svg11.dtd\">\n<svg><rect/></svg>";

    assert_eq!(format_of(plain).unwrap(), PictureFormat::Svg);
    assert_eq!(format_of(declared).unwrap(), PictureFormat::Svg);
}

#[test]
fn anything_else_is_refused_as_the_callers_mistake() {
    let cases: [&[u8]; 5] = [b"just some text", b"<html><svg/></html>", b"<svg><unclosed></svg>", b"BM bitmap", b""];

    for bytes in cases {
        assert!(matches!(format_of(bytes), Err(RekallError::IllegalArgument(_))), "{bytes:?}");
    }
}
