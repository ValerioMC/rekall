use super::*;
use image::Rgba;

fn png(image: &RgbaImage) -> Vec<u8> {
    encode(image).unwrap()
}

fn decoded(bytes: &[u8]) -> RgbaImage {
    decode(bytes).unwrap()
}

#[test]
fn a_wide_upload_becomes_the_centred_square_of_its_shorter_side() {
    let wide = RgbaImage::from_fn(300, 100, |x, _| match x {
        0..=99 => Rgba([255, 0, 0, 255]),
        100..=199 => Rgba([0, 255, 0, 255]),
        _ => Rgba([0, 0, 255, 255]),
    });

    let master = decoded(&normalise(&png(&wide)).unwrap());

    assert_eq!((master.width(), master.height()), (100, 100), "a small upload is not enlarged");
    assert_eq!(master.get_pixel(50, 50), &Rgba([0, 255, 0, 255]), "the middle third is what is kept");
}

#[test]
fn a_large_upload_is_shrunk_to_the_master_side() {
    let large = RgbaImage::from_pixel(1_024, 600, Rgba([10, 20, 30, 255]));

    let master = decoded(&normalise(&png(&large)).unwrap());

    assert_eq!((master.width(), master.height()), (MASTER_SIDE, MASTER_SIDE));
}

#[test]
fn a_render_is_exactly_the_size_asked_and_never_larger_than_the_master() {
    let master = png(&RgbaImage::from_pixel(256, 256, Rgba([200, 100, 50, 255])));

    let small = decoded(&render(&master, 28).unwrap());
    let too_small = decoded(&render(&master, 1).unwrap());
    let too_large = decoded(&render(&master, 500).unwrap());

    assert_eq!((small.width(), small.height()), (28, 28));
    assert_eq!(too_small.width(), MIN_RENDER_SIDE);
    assert_eq!(too_large.width(), 256, "the master is returned rather than stretched");
}

#[test]
fn a_flat_colour_survives_a_resize_unchanged() {
    let master = png(&RgbaImage::from_pixel(512, 512, Rgba([200, 100, 50, 255])));

    let render = decoded(&render(&master, 34).unwrap());

    assert!(render.pixels().all(|pixel| pixel == &Rgba([200, 100, 50, 255])));
}

#[test]
fn colour_hidden_under_transparency_does_not_fringe_the_edge() {
    let logo = RgbaImage::from_fn(256, 256, |x, _| if x < 128 { Rgba([255, 255, 255, 255]) } else { Rgba([0, 0, 0, 0]) });

    let render = decoded(&render(&png(&logo), 24).unwrap());

    let visible: Vec<&Rgba<u8>> = render.pixels().filter(|pixel| pixel[3] > 0).collect();
    assert!(!visible.is_empty());
    assert!(visible.iter().all(|pixel| pixel[0] >= 250 && pixel[1] >= 250 && pixel[2] >= 250), "{visible:?}");
}

#[test]
fn a_file_that_is_not_an_image_is_refused_as_bad_input() {
    let error = normalise(b"definitely not a picture").unwrap_err();

    assert!(matches!(error, RekallError::IllegalArgument(_)), "{error:?}");
}
