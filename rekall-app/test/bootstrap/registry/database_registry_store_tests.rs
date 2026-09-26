use crate::bootstrap::registry::DatabaseEntry;

use super::*;

fn entry(id: &str, label: &str, path: &str) -> DatabaseEntry {
    DatabaseEntry { id: id.into(), label: label.into(), path: path.into(), added_at: "t".into(), last_used_at: "t".into() }
}

#[test]
fn a_registry_that_was_never_written_reads_as_empty() {
    let home = tempfile::tempdir().unwrap();
    assert_eq!(DatabaseRegistryStore::new(home.path()).read().unwrap(), None);
}

#[test]
fn what_is_written_is_what_comes_back() {
    let home = tempfile::tempdir().unwrap();
    let store = DatabaseRegistryStore::new(home.path());
    let first = DatabaseEntry {
        id: "id-1".into(),
        label: "Local".into(),
        path: "/tmp/rekall".into(),
        added_at: "2026-01-01T00:00:00Z".into(),
        last_used_at: "2026-01-01T00:00:00Z".into(),
    };
    store.write(&DatabaseRegistry { active_id: Some("id-1".into()), databases: vec![first.clone()] }).unwrap();

    let read = store.read().unwrap().unwrap();
    assert_eq!(read.active_id.as_deref(), Some("id-1"));
    assert_eq!(read.active(), Some(&first));
}

#[test]
fn writing_twice_overwrites_rather_than_appending() {
    let home = tempfile::tempdir().unwrap();
    let store = DatabaseRegistryStore::new(home.path());
    store.write(&DatabaseRegistry { active_id: Some("a".into()), databases: vec![entry("a", "First", "/a")] }).unwrap();
    store.write(&DatabaseRegistry { active_id: Some("b".into()), databases: vec![entry("b", "Second", "/b")] }).unwrap();

    let read = store.read().unwrap().unwrap();
    assert_eq!(read.databases.len(), 1);
    assert_eq!(read.active_id.as_deref(), Some("b"));
}

#[test]
fn the_home_directory_is_created_on_first_write() {
    let home = tempfile::tempdir().unwrap();
    let nested = home.path().join("nested/.rekall");
    DatabaseRegistryStore::new(&nested)
        .write(&DatabaseRegistry { active_id: Some("a".into()), databases: vec![entry("a", "First", "/a")] })
        .unwrap();
    assert!(nested.join("config.json").exists());
}

#[test]
fn an_entry_with_no_matching_active_id_is_reported_as_no_active_entry() {
    let home = tempfile::tempdir().unwrap();
    let store = DatabaseRegistryStore::new(home.path());
    store
        .write(&DatabaseRegistry { active_id: Some("missing".into()), databases: vec![entry("a", "First", "/a")] })
        .unwrap();
    assert_eq!(store.read().unwrap().unwrap().active(), None);
}

#[test]
fn a_file_the_java_build_wrote_reads_back() {
    let home = tempfile::tempdir().unwrap();
    std::fs::write(
        home.path().join("config.json"),
        "{\n  \"activeId\" : \"a\",\n  \"databases\" : [ {\n    \"id\" : \"a\",\n    \"label\" : \"Local\",\n    \
         \"path\" : \"/x\",\n    \"addedAt\" : \"t1\",\n    \"lastUsedAt\" : \"t2\"\n  } ]\n}",
    )
    .unwrap();
    let read = DatabaseRegistryStore::new(home.path()).read().unwrap().unwrap();
    assert_eq!(read.active().unwrap().last_used_at, "t2");
}
