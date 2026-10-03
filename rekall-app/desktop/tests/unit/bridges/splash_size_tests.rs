use super::*;

#[test]
fn a_laptop_screen_gets_the_minimum_width() {
    let size = splash_size(2880, 2.0);
    assert_eq!((size.width, size.height), (640.0, 357.0));
}

#[test]
fn a_full_hd_screen_gets_two_fifths_of_its_width() {
    let size = splash_size(1920, 1.0);
    assert_eq!((size.width, size.height), (768.0, 429.0));
}

#[test]
fn a_large_screen_is_capped_at_the_maximum_width() {
    let size = splash_size(5120, 2.0);
    assert_eq!((size.width, size.height), (960.0, 536.0));
}

#[test]
fn the_artwork_proportions_are_kept() {
    let size = splash_size(3440, 1.0);
    assert!((size.width / size.height - ASPECT_RATIO).abs() < 0.01);
}
