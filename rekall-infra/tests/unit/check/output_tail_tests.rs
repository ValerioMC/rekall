use super::*;

#[test]
fn short_output_is_kept_whole() {
    assert_eq!(output_tail(b"one\ntwo\n"), "one\ntwo");
}

#[test]
fn only_the_last_sixty_lines_are_kept() {
    let output: String = (1..=100).map(|number| format!("line {number}\n")).collect();

    let tail = output_tail(output.as_bytes());

    assert_eq!(tail.lines().count(), 60);
    assert_eq!(tail.lines().next(), Some("line 41"));
    assert_eq!(tail.lines().last(), Some("line 100"));
}

#[test]
fn a_very_long_line_is_cut_from_its_start() {
    let output = format!("{}END", "x".repeat(10_000));

    let tail = output_tail(output.as_bytes());

    assert_eq!(tail.chars().count(), 4_000);
    assert!(tail.ends_with("xxEND"));
}

#[test]
fn bytes_that_are_not_utf8_are_replaced_instead_of_failing() {
    assert_eq!(output_tail(&[b'o', b'k', 0xff, b'!']), "ok\u{fffd}!");
}

#[test]
fn no_output_is_an_empty_tail() {
    assert_eq!(output_tail(b""), "");
}
