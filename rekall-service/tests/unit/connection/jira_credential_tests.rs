use super::*;

#[test]
fn the_store_keeps_the_pair_basic_authentication_sends() {
    let credential = JiraCredential::new(" ada@example.com ", Secret::new("tok3n")).unwrap();
    assert_eq!(credential.to_stored().expose(), "ada@example.com:tok3n");
    assert_eq!(credential.authorization().expose(), format!("Basic {}", STANDARD.encode("ada@example.com:tok3n")));
}

#[test]
fn a_stored_credential_reads_back_with_a_token_that_holds_a_colon() {
    let credential = JiraCredential::from_stored(&Secret::new("ada@example.com:a:b")).unwrap();
    assert_eq!(credential.email(), "ada@example.com");
    assert_eq!(credential.to_stored().expose(), "ada@example.com:a:b");
}

#[test]
fn an_email_that_is_not_one_or_that_holds_a_colon_is_refused() {
    assert!(JiraCredential::new("ada", Secret::new("t")).is_err());
    assert!(JiraCredential::new("a:da@example.com", Secret::new("t")).is_err());
}

#[test]
fn a_blank_token_is_refused() {
    assert!(JiraCredential::new("ada@example.com", Secret::new("  ")).is_err());
}

#[test]
fn a_stored_value_without_a_pair_is_corrupt() {
    assert!(JiraCredential::from_stored(&Secret::new("nothing")).is_err());
}
