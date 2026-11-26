use rekall_core::picture::PicturePath;

use super::*;

fn picture(relative: &str, modified_at: Instant) -> StoredPicture {
    StoredPicture { path: PicturePath::parse(relative).unwrap(), size_bytes: 1, modified_at }
}

#[test]
fn a_file_is_settled_a_day_after_it_was_written() {
    let now = Instant::parse("2026-03-10T12:00:00Z").unwrap();
    let written = |at: &str| picture("acme/web/web-12/a.png", Instant::parse(at).unwrap());

    assert!(is_settled(&written("2026-03-09T12:00:00Z"), now));
    assert!(!is_settled(&written("2026-03-09T12:00:01Z"), now));
    assert!(!is_settled(&written("2026-03-11T00:00:00Z"), now));
}

#[test]
fn a_link_followed_by_anything_still_shows_its_picture() {
    let mut unused = vec![picture("acme/web/web-12/a.png", Instant::EPOCH), picture("acme/web/web-12/b.png", Instant::EPOCH)];

    forget_shown(&mut unused, &["see ![a](/api/images/acme/web/web-12/a.png#right).".to_string(), "acme/web/web-12/b.png".to_string()]);

    assert_eq!(unused, [picture("acme/web/web-12/b.png", Instant::EPOCH)]);
}

#[test]
fn progress_is_a_whole_number_that_stops_at_a_hundred() {
    assert_eq!(percentage(0, 3), 0);
    assert_eq!(percentage(2, 3), 66);
    assert_eq!(percentage(5, 3), 100);
    assert_eq!(percentage(0, 0), 100);
}
