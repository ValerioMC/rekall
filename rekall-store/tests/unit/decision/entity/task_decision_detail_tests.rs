use super::*;

#[test]
fn a_detail_over_its_cap_is_refused() {
    let detail = Model::new(Id::random(), "d".repeat(TEXT_MAX_CHARACTERS + 1), Instant::now());
    assert!(detail.validate().unwrap_err().message().contains("text is capped at 2000"));
}

#[test]
fn a_detail_with_text_passes() {
    assert!(Model::new(Id::random(), "Rotate it every 90 days.".into(), Instant::now()).validate().is_ok());
}
