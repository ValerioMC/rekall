use super::*;

#[test]
fn a_service_account_token_is_kept_bare_and_sent_as_bearer() {
    let credential = GrafanaCredential::service_account_token(Secret::new("glsa_abc")).unwrap();
    assert_eq!(credential.to_stored().expose(), "glsa_abc");
    assert_eq!(credential.authorization().expose(), "Bearer glsa_abc");
    assert_eq!(credential.user(), None);
}

#[test]
fn a_user_and_password_are_kept_as_the_pair_basic_authentication_sends() {
    let credential = GrafanaCredential::password(" admin ", Secret::new("s3cret")).unwrap();
    assert_eq!(credential.to_stored().expose(), "admin:s3cret");
    assert_eq!(credential.authorization().expose(), format!("Basic {}", STANDARD.encode("admin:s3cret")));
    assert_eq!(credential.user(), Some("admin"));
}

#[test]
fn a_stored_value_with_a_colon_reads_back_as_a_password_that_may_hold_colons() {
    let credential = GrafanaCredential::from_stored(&Secret::new("admin:a:b")).unwrap();
    assert_eq!(credential.user(), Some("admin"));
    assert_eq!(credential.secret().expose(), "a:b");
    assert_eq!(credential.to_stored().expose(), "admin:a:b");
}

#[test]
fn a_stored_value_without_a_colon_reads_back_as_a_token() {
    let credential = GrafanaCredential::from_stored(&Secret::new("glsa_abc")).unwrap();
    assert_eq!(credential.user(), None);
    assert_eq!(credential.authorization().expose(), "Bearer glsa_abc");
}

#[test]
fn a_blank_secret_a_blank_user_or_a_colon_in_the_token_or_user_is_refused() {
    assert!(GrafanaCredential::service_account_token(Secret::new("  ")).is_err());
    assert!(GrafanaCredential::service_account_token(Secret::new("a:b")).is_err());
    assert!(GrafanaCredential::password("", Secret::new("p")).is_err());
    assert!(GrafanaCredential::password("a:b", Secret::new("p")).is_err());
    assert!(GrafanaCredential::password("admin", Secret::new("  ")).is_err());
}
