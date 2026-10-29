use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;
use crate::connection::InMemorySecretStore;

/// The wrapped store, counting how often it is read: each read is a keychain prompt.
#[derive(Default)]
struct CountingStore {
    store: InMemorySecretStore,
    reads: AtomicUsize,
}

impl CountingStore {
    fn reads(&self) -> usize {
        self.reads.load(Ordering::SeqCst)
    }
}

impl SecretStore for CountingStore {
    fn put(&self, account: &str, secret: &Secret) -> Result<()> {
        self.store.put(account, secret)
    }

    fn read(&self, account: &str) -> Result<Secret> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.store.read(account)
    }

    fn remove(&self, account: &str) -> Result<()> {
        self.store.remove(account)
    }
}

fn remembering(counting: &Arc<CountingStore>) -> RememberingSecretStore {
    RememberingSecretStore::new(counting.clone())
}

#[test]
fn an_account_is_read_from_the_wrapped_store_once() {
    let counting = Arc::new(CountingStore::default());
    counting.put("connection-1", &Secret::new("token")).unwrap();
    let store = remembering(&counting);
    assert_eq!(store.read("connection-1").unwrap().expose(), "token");
    assert_eq!(store.read("connection-1").unwrap().expose(), "token");
    assert_eq!(counting.reads(), 1);
}

#[test]
fn a_put_is_written_through_and_read_back_without_asking_the_wrapped_store() {
    let counting = Arc::new(CountingStore::default());
    let store = remembering(&counting);
    store.put("connection-1", &Secret::new("old")).unwrap();
    store.put("connection-1", &Secret::new("new")).unwrap();
    assert_eq!(store.read("connection-1").unwrap().expose(), "new");
    assert_eq!(counting.store.read("connection-1").unwrap().expose(), "new");
    assert_eq!(counting.reads(), 0);
}

#[test]
fn a_removed_account_is_forgotten_and_removed_from_the_wrapped_store() {
    let counting = Arc::new(CountingStore::default());
    let store = remembering(&counting);
    store.put("connection-1", &Secret::new("token")).unwrap();
    store.remove("connection-1").unwrap();
    assert!(matches!(store.read("connection-1").unwrap_err(), RekallError::NotFound(_)));
    assert!(!counting.store.contains("connection-1"));
}

#[test]
fn an_account_with_nothing_stored_is_asked_again_on_the_next_read() {
    let counting = Arc::new(CountingStore::default());
    let store = remembering(&counting);
    assert!(store.read("connection-1").is_err());
    counting.put("connection-1", &Secret::new("token")).unwrap();
    assert_eq!(store.read("connection-1").unwrap().expose(), "token");
    assert_eq!(counting.reads(), 2);
}
