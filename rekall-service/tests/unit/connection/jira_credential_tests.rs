use super::*;

#[test]
fn a_cloud_credential_is_kept_as_the_pair_basic_authentication_sends() {
    let credential = JiraCredential::cloud(" ada@example.com ", Secret::new("tok3n")).unwrap();
    assert_eq!(credential.to_stored().expose(), "ada@example.com:tok3n");
    assert_eq!(credential.authorization().expose(), format!("Basic {}", STANDARD.encode("ada@example.com:tok3n")));
    assert_eq!(credential.deployment(), JiraDeployment::Cloud);
}

#[test]
fn a_data_center_credential_sends_its_personal_access_token_as_bearer() {
    let credential = JiraCredential::data_center(Secret::new("NjQ0Mj")).unwrap();
    assert_eq!(credential.to_stored().expose(), "pat:NjQ0Mj");
    assert_eq!(credential.authorization().expose(), "Bearer NjQ0Mj");
    assert_eq!(credential.deployment(), JiraDeployment::DataCenter);
    assert_eq!(credential.email(), None);
}

#[test]
fn a_stored_credential_reads_back_with_a_token_that_holds_a_colon() {
    let credential = JiraCredential::from_stored(&Secret::new("ada@example.com:a:b")).unwrap();
    assert_eq!(credential.email(), Some("ada@example.com"));
    assert_eq!(credential.to_stored().expose(), "ada@example.com:a:b");
}

#[test]
fn a_stored_personal_access_token_reads_back_as_data_center() {
    let credential = JiraCredential::from_stored(&Secret::new("pat:NjQ0Mj")).unwrap();
    assert_eq!(credential.deployment(), JiraDeployment::DataCenter);
    assert_eq!(credential.authorization().expose(), "Bearer NjQ0Mj");
}

#[test]
fn an_email_that_is_not_one_or_that_holds_a_colon_is_refused() {
    assert!(JiraCredential::cloud("ada", Secret::new("t")).is_err());
    assert!(JiraCredential::cloud("a:da@example.com", Secret::new("t")).is_err());
}

#[test]
fn a_blank_token_is_refused() {
    assert!(JiraCredential::cloud("ada@example.com", Secret::new("  ")).is_err());
    assert!(JiraCredential::data_center(Secret::new("  ")).is_err());
}

#[test]
fn a_stored_value_that_is_neither_shape_is_corrupt() {
    assert!(JiraCredential::from_stored(&Secret::new("nothing")).is_err());
    assert!(JiraCredential::from_stored(&Secret::new("vcasale:secret")).is_err());
}
