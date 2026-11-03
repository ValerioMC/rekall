use super::*;
use crate::connection::InMemorySecretStore;

struct Stores {
    current: Arc<InMemorySecretStore>,
    previous: Arc<InMemorySecretStore>,
    migrating: MigratingSecretStore,
}

fn stores() -> Stores {
    let current = Arc::new(InMemorySecretStore::new());
    let previous = Arc::new(InMemorySecretStore::new());
    let migrating = MigratingSecretStore::new(current.clone(), previous.clone());
    Stores { current, previous, migrating }
}

#[test]
fn a_credential_only_the_previous_store_holds_is_read_and_copied_to_the_current_one() {
    let stores = stores();
    stores.previous.put("connection-1", &Secret::new("token")).unwrap();
    assert_eq!(stores.migrating.read("connection-1").unwrap().expose(), "token");
    assert_eq!(stores.current.read("connection-1").unwrap().expose(), "token");
}

#[test]
fn a_credential_the_current_store_holds_is_read_without_asking_the_previous_one() {
    let stores = stores();
    stores.current.put("connection-1", &Secret::new("new")).unwrap();
    stores.previous.put("connection-1", &Secret::new("old")).unwrap();
    assert_eq!(stores.migrating.read("connection-1").unwrap().expose(), "new");
}

#[test]
fn an_account_neither_store_holds_is_not_found() {
    let stores = stores();
    assert!(matches!(stores.migrating.read("connection-1"), Err(RekallError::NotFound(_))));
}

#[test]
fn a_put_goes_to_the_current_store_only() {
    let stores = stores();
    stores.migrating.put("connection-1", &Secret::new("token")).unwrap();
    assert!(stores.current.contains("connection-1"));
    assert!(!stores.previous.contains("connection-1"));
}

#[test]
fn a_removed_credential_does_not_come_back_from_the_previous_store() {
    let stores = stores();
    stores.previous.put("connection-1", &Secret::new("token")).unwrap();
    stores.migrating.read("connection-1").unwrap();
    stores.migrating.remove("connection-1").unwrap();
    assert!(matches!(stores.migrating.read("connection-1"), Err(RekallError::NotFound(_))));
}
