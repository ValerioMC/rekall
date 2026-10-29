use super::*;

#[test]
fn a_row_that_breaks_nothing_passes() {
    let checked = Violations::new("Task")
        .not_blank("title", Some("Report"))
        .max_length("title", Some("Report"), 200)
        .slug("label", Some("report-builder"))
        .finish();
    assert!(checked.is_ok());
}

#[test]
fn every_broken_rule_is_reported_together_as_the_callers_mistake() {
    let refused = Violations::new("Task")
        .not_blank("title", Some("   "))
        .slug("label", Some("Report Builder"))
        .finish()
        .unwrap_err();
    assert!(refused.is_illegal_argument());
    assert_eq!(
        refused.message(),
        "Task: title must not be blank; label must be a slug: lowercase letters, digits, '-', '_' or '.'."
    );
}

#[test]
fn a_length_cap_counts_utf16_units_and_says_by_how_much_it_was_passed() {
    let refused = Violations::new("Tag").max_length("name", Some("😀😀"), 3).finish().unwrap_err();
    assert_eq!(refused.message(), "Tag: name is capped at 3 characters and this one is 4.");
}

#[test]
fn an_absent_optional_value_has_no_length_to_check() {
    assert!(Violations::new("Project").max_length("description", None, 10).finish().is_ok());
}
