use super::*;

#[test]
fn a_folder_is_stored_without_its_trailing_separator() {
    assert_eq!(folder(Some("  /work/api/ ")).unwrap(), "/work/api");
    assert_eq!(folder(Some("services\\billing\\")).unwrap(), "services\\billing");
}

#[test]
fn the_root_folder_is_kept_whole() {
    assert_eq!(folder(Some("/")).unwrap(), "/");
}

#[test]
fn a_missing_or_blank_folder_is_refused_as_a_client_mistake() {
    assert!(folder(None).is_err());
    assert!(folder(Some("   ")).is_err());
}

#[test]
fn a_summary_a_session_writes_must_be_there_and_short() {
    assert!(written_summary(None).unwrap_err().message().contains("needs a summary"));
    assert!(written_summary(Some(&"s".repeat(501))).unwrap_err().message().contains("capped at 500"));
    assert_eq!(written_summary(Some("  Charges customers. ")).unwrap(), "Charges customers.");
}

#[test]
fn two_names_are_the_same_ignoring_case_and_surrounding_spaces() {
    assert!(is_same_name(" Billing API", "billing api "));
    assert!(!is_same_name("Billing", "Billing API"));
}
