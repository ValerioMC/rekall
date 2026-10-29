use super::*;

#[test]
fn blank_and_absent_text_read_as_absent() {
    assert_eq!(present(None), None);
    assert_eq!(present(Some(" \n\t")), None);
}

#[test]
fn present_text_is_stripped() {
    assert_eq!(present(Some("  a title \n")), Some("a title"));
}

#[test]
fn text_at_the_cap_is_kept() {
    assert_eq!(within("abcde", 5, |_| unreachable!()).unwrap(), "abcde");
}

#[test]
fn text_past_the_cap_is_refused_with_the_length_found() {
    let refused = within("abcdef", 5, |length| format!("too long: {length}")).unwrap_err();
    assert!(refused.is_illegal_argument());
    assert_eq!(refused.message(), "too long: 6");
}

#[test]
fn length_counts_utf16_units_as_the_console_does() {
    assert!(within("😀😀", 3, |length| length.to_string()).is_err());
    assert!(within("èèè", 3, |_| unreachable!()).is_ok());
}
