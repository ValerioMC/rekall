//! `~/.rekall/config.json`: every database folder a person has registered and which one is in
//! use. The file is shared with the Java build, which reads it with Jackson, so the shape is the
//! records' own: `{"activeId": ..., "databases": [{"id", "label", "path", "addedAt", "lastUsedAt"}]}`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseEntry {
    pub id: String,
    pub label: String,
    pub path: String,
    pub added_at: String,
    pub last_used_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseRegistry {
    pub active_id: Option<String>,
    #[serde(default)]
    pub databases: Vec<DatabaseEntry>,
}

impl DatabaseRegistry {
    pub fn empty() -> Self {
        Self { active_id: None, databases: Vec::new() }
    }

    pub fn active(&self) -> Option<&DatabaseEntry> {
        let active = self.active_id.as_deref()?;
        self.databases.iter().find(|entry| entry.id == active)
    }
}

#[derive(Clone, Debug)]
pub struct DatabaseRegistryStore {
    config_file: PathBuf,
}

impl DatabaseRegistryStore {
    /// `rekall.home`, `~/.rekall` unless set.
    pub fn new(home: &Path) -> Self {
        Self { config_file: home.join("config.json") }
    }

    pub fn read(&self) -> Result<Option<DatabaseRegistry>, String> {
        if !self.config_file.is_file() {
            return Ok(None);
        }
        let text = std::fs::read(&self.config_file).map_err(|_| self.unreadable())?;
        serde_json::from_slice(&text).map(Some).map_err(|_| self.unreadable())
    }

    pub fn write(&self, registry: &DatabaseRegistry) -> Result<(), String> {
        let failed = || format!("Could not write {}", self.config_file.display());
        if let Some(parent) = self.config_file.parent() {
            std::fs::create_dir_all(parent).map_err(|_| failed())?;
        }
        let text = serde_json::to_string_pretty(registry).map_err(|_| failed())?;
        std::fs::write(&self.config_file, text).map_err(|_| failed())
    }

    fn unreadable(&self) -> String {
        format!("Could not read {}", self.config_file.display())
    }
}

#[cfg(test)]
mod tests {
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
}
