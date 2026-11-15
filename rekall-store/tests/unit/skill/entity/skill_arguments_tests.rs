use super::*;

#[test]
fn arguments_are_trimmed() {
    assert_eq!(normalize(Some("  redesign website ui \n")).as_deref(), Some("redesign website ui"));
}

#[test]
fn blank_arguments_are_none() {
    assert_eq!(normalize(Some("   ")), None);
    assert_eq!(normalize(Some("")), None);
    assert_eq!(normalize(None), None);
}

#[test]
fn arguments_up_to_the_cap_are_accepted() {
    assert!(validate(Some(&"a".repeat(SKILL_ARGUMENTS_MAX))).is_ok());
}

#[test]
fn arguments_over_the_cap_are_refused() {
    let failure = validate(Some(&"a".repeat(SKILL_ARGUMENTS_MAX + 1))).unwrap_err();
    assert!(failure.is_illegal_argument());
}
