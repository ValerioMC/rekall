use super::*;

#[test]
fn strip_keeps_a_non_breaking_space_like_java() {
    assert_eq!(strip("\u{00A0}x "), "\u{00A0}x");
    assert!(!is_blank("\u{00A0}"));
    assert!(is_blank(" \t\n"));
}

#[test]
fn length_counts_utf16_units() {
    assert_eq!(len("abc"), 3);
    assert_eq!(len("é"), 1);
    assert_eq!(len("😀"), 2);
    assert_eq!(prefix("a😀b", 2), "a");
    assert_eq!(prefix("a😀b", 3), "a😀");
}

#[test]
fn lines_split_like_java() {
    assert_eq!(lines("a\nb\r\nc\rd\n"), vec!["a", "b", "c", "d"]);
    assert_eq!(lines(""), Vec::<&str>::new());
    assert_eq!(lines("\n\nx"), vec!["", "", "x"]);
}

#[test]
fn split_on_line_breaks_drops_trailing_empties_unless_asked() {
    assert_eq!(split_linebreaks("a\n\nb\n\n", false), vec!["a", "", "b"]);
    assert_eq!(split_linebreaks("a\n\nb\n", true), vec!["a", "", "b", ""]);
    assert_eq!(split_linebreaks("a\r\nb", false), vec!["a", "b"]);
}

#[test]
fn collapses_ascii_whitespace_only() {
    assert_eq!(collapse_whitespace("a \t\n b\u{00A0}c"), "a b\u{00A0}c");
}

#[test]
fn finds_like_java() {
    assert_eq!(index_of("héllo wörld", "wörld"), Some(6));
    assert_eq!(last_index_of_from("a b c d", " ", 4), Some(3));
    assert_eq!(index_of_from("a b c d", " ", 2), Some(3));
    assert!(!equals_ignore_case("Straße", "STRASSE"));
    assert!(equals_ignore_case("Hello", "hELLO"));
}

#[test]
fn orders_ignoring_case_like_java() {
    use std::cmp::Ordering::*;
    assert_eq!(compare_ignore_case("app.ts", "README.md"), Less);
    assert_eq!(compare_ignore_case("Docs", "docs"), Equal);
    assert_eq!(compare_ignore_case("src", "Src2"), Less);
    assert_eq!(compare_ignore_case(".env", "app"), Less);
}
