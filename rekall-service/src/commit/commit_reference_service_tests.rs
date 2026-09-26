use super::*;

#[test]
fn an_overlong_subject_is_cut_to_the_comment_cap() {
    let cut = truncated(&"x".repeat(COMMENT_MAX + 50));
    assert_eq!(jstr::len(&cut), COMMENT_MAX);
    assert!(cut.ends_with('…'));
    assert_eq!(truncated("   "), "(no commit message)");
}

#[test]
fn an_overlong_diff_is_truncated_rather_than_stored_whole() {
    let cut = truncated_diff(Some(&"+".repeat(DIFF_MAX + 500))).unwrap();
    assert_eq!(jstr::len(&cut), DIFF_MAX + jstr::len("\n…(diff truncated)"));
    assert!(cut.ends_with("…(diff truncated)"));
    assert_eq!(truncated_diff(None), None);
    assert_eq!(truncated_diff(Some("  ")), None);
}
