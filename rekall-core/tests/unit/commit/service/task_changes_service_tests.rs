use super::*;

#[test]
fn a_patch_within_the_cap_is_kept_whole() {
    let (kept, truncated) = capped("+short\n".to_string());

    assert_eq!(kept, "+short\n");
    assert!(!truncated);
}

#[test]
fn a_patch_past_the_cap_is_cut_on_a_character_and_flagged() {
    let (kept, truncated) = capped("é".repeat(PATCH_MAX + 10));

    assert_eq!(kept.chars().count(), PATCH_MAX);
    assert!(truncated);
}
