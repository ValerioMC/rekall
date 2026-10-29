use super::*;

#[test]
fn a_stored_secret_reads_back() {
    let store = InMemorySecretStore::new();
    store.put("connection-1", &Secret::new("token")).unwrap();
    assert_eq!(store.read("connection-1").unwrap().expose(), "token");
    assert!(store.contains("connection-1"));
}

#[test]
fn reading_an_account_with_nothing_stored_is_not_found() {
    let error = InMemorySecretStore::new().read("connection-1").unwrap_err();
    assert!(matches!(error, RekallError::NotFound(_)));
}

#[test]
fn a_put_replaces_what_the_account_held() {
    let store = InMemorySecretStore::new();
    store.put("connection-1", &Secret::new("old")).unwrap();
    store.put("connection-1", &Secret::new("new")).unwrap();
    assert_eq!(store.read("connection-1").unwrap().expose(), "new");
}

#[test]
fn removing_succeeds_whether_or_not_something_was_stored() {
    let store = InMemorySecretStore::new();
    store.put("connection-1", &Secret::new("token")).unwrap();
    store.remove("connection-1").unwrap();
    store.remove("connection-1").unwrap();
    assert!(!store.contains("connection-1"));
}
