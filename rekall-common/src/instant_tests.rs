use super::*;

fn at(text: &str) -> Instant {
    Instant::parse(text).unwrap()
}

#[test]
fn prints_the_way_java_prints_an_instant() {
    assert_eq!(at("2024-05-01T10:15:30Z").to_string(), "2024-05-01T10:15:30Z");
    assert_eq!(at("2024-05-01T10:15:30.100Z").to_string(), "2024-05-01T10:15:30.100Z");
    assert_eq!(at("2024-05-01T10:15:30.123456Z").to_string(), "2024-05-01T10:15:30.123456Z");
    assert_eq!(at("2024-05-01T10:15:30.120001Z").to_string(), "2024-05-01T10:15:30.120001Z");
}

#[test]
fn keeps_microseconds_and_no_more() {
    assert_eq!(at("2024-05-01T10:15:30.123456789Z").to_string(), "2024-05-01T10:15:30.123456Z");
}

#[test]
fn reads_an_offset_as_the_same_instant() {
    assert_eq!(at("2024-05-01T12:15:30+02:00"), at("2024-05-01T10:15:30Z"));
}

#[test]
fn the_stored_text_sorts_in_time_order() {
    let earlier = at("2024-05-01T10:15:30Z").to_db_string();
    let later = at("2024-05-01T10:15:30.000001Z").to_db_string();
    assert_eq!(earlier, "2024-05-01T10:15:30.000000Z");
    assert!(earlier < later);
}
