use super::*;

fn store_in(folder: &Path) -> FileSecretStore {
    FileSecretStore::new(folder.join("secrets.json"))
}

#[test]
fn a_stored_credential_is_read_back() {
    let folder = tempfile::tempdir().unwrap();
    let store = store_in(folder.path());
    store.put("connection-1", &Secret::new("token")).unwrap();
    assert_eq!(store.read("connection-1").unwrap().expose(), "token");
}

#[test]
fn a_credential_survives_a_new_store_on_the_same_file() {
    let folder = tempfile::tempdir().unwrap();
    store_in(folder.path()).put("connection-1", &Secret::new("token")).unwrap();
    assert_eq!(store_in(folder.path()).read("connection-1").unwrap().expose(), "token");
}

#[test]
fn an_account_never_stored_is_not_found() {
    let folder = tempfile::tempdir().unwrap();
    assert!(matches!(store_in(folder.path()).read("connection-1"), Err(RekallError::NotFound(_))));
}

#[test]
fn a_put_replaces_the_value_and_keeps_the_other_accounts() {
    let folder = tempfile::tempdir().unwrap();
    let store = store_in(folder.path());
    store.put("connection-1", &Secret::new("old")).unwrap();
    store.put("connection-2", &Secret::new("other")).unwrap();
    store.put("connection-1", &Secret::new("new")).unwrap();
    assert_eq!(store.read("connection-1").unwrap().expose(), "new");
    assert_eq!(store.read("connection-2").unwrap().expose(), "other");
}

#[test]
fn a_removed_credential_is_gone_and_removing_a_missing_one_succeeds() {
    let folder = tempfile::tempdir().unwrap();
    let store = store_in(folder.path());
    store.put("connection-1", &Secret::new("token")).unwrap();
    store.remove("connection-1").unwrap();
    store.remove("connection-1").unwrap();
    assert!(matches!(store.read("connection-1"), Err(RekallError::NotFound(_))));
}

#[test]
fn the_folder_of_the_file_is_created() {
    let folder = tempfile::tempdir().unwrap();
    let store = FileSecretStore::new(folder.path().join("home").join("secrets.json"));
    store.put("connection-1", &Secret::new("token")).unwrap();
    assert_eq!(store.read("connection-1").unwrap().expose(), "token");
}

#[test]
fn a_file_that_is_not_json_is_an_internal_failure_not_an_empty_store() {
    let folder = tempfile::tempdir().unwrap();
    fs::write(folder.path().join("secrets.json"), "not json").unwrap();
    assert!(matches!(store_in(folder.path()).read("connection-1"), Err(RekallError::Internal(_))));
}

#[cfg(unix)]
#[test]
fn the_file_is_readable_by_its_owner_only() {
    use std::os::unix::fs::PermissionsExt;
    let folder = tempfile::tempdir().unwrap();
    store_in(folder.path()).put("connection-1", &Secret::new("token")).unwrap();
    let mode = fs::metadata(folder.path().join("secrets.json")).unwrap().permissions().mode();
    assert_eq!(mode & 0o077, 0);
}
