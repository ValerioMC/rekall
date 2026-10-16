use super::length_nudge;

#[test]
fn a_wrapup_within_the_word_budget_carries_no_nudge() {
    assert_eq!(length_nudge(&"word ".repeat(600)), None);
}

#[test]
fn a_wrapup_past_the_word_budget_is_told_its_length() {
    let nudge = length_nudge(&"word ".repeat(601)).expect("a nudge past 600 words");
    assert!(nudge.starts_with("It is 601 words"), "{nudge}");
}
