use super::*;

#[test]
fn a_space_key_is_kept_as_typed_and_a_personal_one_keeps_its_tilde() {
    assert_eq!(space_key_of(Some(" ENG ")).unwrap(), "ENG");
    assert_eq!(space_key_of(Some("~ada.lovelace")).unwrap(), "~ada.lovelace");
}

#[test]
fn a_blank_target_or_one_with_spaces_or_slashes_is_not_a_space_key() {
    assert!(space_key_of(None).is_err());
    assert!(space_key_of(Some("  ")).is_err());
    assert!(space_key_of(Some("~")).is_err());
    assert!(space_key_of(Some("my space")).is_err());
    assert!(space_key_of(Some("ENG/Home")).is_err());
}
