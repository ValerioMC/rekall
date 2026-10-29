use super::*;

#[test]
fn an_excerpt_surrounds_the_match_on_one_line_cut_at_a_word() {
    let text = format!("{}the\nsettlement   batch runs nightly {}", "word ".repeat(30), "tail ".repeat(30));
    let excerpt = excerpt(Some(&text), "Settlement batch");
    assert!(excerpt.starts_with("…word"), "{excerpt}");
    assert!(excerpt.ends_with('…'));
    assert!(excerpt.contains("the settlement batch runs nightly"));
    assert!(!excerpt.contains('\n'));
    assert!(!excerpt.contains("  "));
    assert!(jstr::len(&excerpt) <= 2 * EXCERPT_RADIUS + "settlement batch".len() + 2);
}

#[test]
fn a_short_text_is_returned_whole_with_no_ellipsis() {
    assert_eq!(excerpt(Some("Wire the export"), "export"), "Wire the export");
}
