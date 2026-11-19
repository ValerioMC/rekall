use super::*;

#[test]
fn a_long_detail_is_cut_at_a_sentence() {
    let detail = "This sentence is long enough to matter. ".repeat(40);
    let body = derived_body(Some(&detail));
    assert!(jstr::len(&body) <= DERIVED_BODY_MAX);
    assert!(body.ends_with("matter."));
}
