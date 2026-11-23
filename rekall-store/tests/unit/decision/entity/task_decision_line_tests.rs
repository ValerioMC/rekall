use super::*;

#[test]
fn a_line_over_its_cap_is_refused() {
    let line = Model::new(Id::random(), 1, Speaker::Claude, "t".repeat(TEXT_MAX_CHARACTERS + 1), Instant::now());
    assert!(line.validate().unwrap_err().message().contains("text is capped at 4000"));
}

#[test]
fn a_blank_line_is_refused() {
    assert!(Model::new(Id::random(), 1, Speaker::Person, "  ".into(), Instant::now()).validate().is_err());
}
