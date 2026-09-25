//! `DatabaseLocationEnvironmentPostProcessor`: which database this start opens, decided before
//! anything else comes up, from the registry in `rekall.home`.
//!
//! - A registry whose active folder is there: that folder's database.
//! - No registry but a `./data` folder holding a database (the layout before the registry): it is
//!   adopted as the first entry, labelled "Local", and opened.
//! - A registry whose active folder is gone (an unplugged drive): an in-memory database, so the
//!   console can come up and offer to pick another.
//! - Nothing at all: an in-memory database and the setup screen.
//!
//! `REKALL_DB_URL` (or `spring.datasource.url`) set by hand overrides the choice, as it did.

use std::path::{Path, PathBuf};

use rekall_common::{Id, Instant};
use rekall_repository::DATA_FILE_NAME;

use super::folder::{absolute_normalised, LEGACY_DATA_FILE_NAME};
use super::registry::{DatabaseEntry, DatabaseRegistry, DatabaseRegistryStore};
use crate::config::{AppConfig, DatabaseOverride};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetupStatus {
    Ready,
    Unreachable,
    SetupNeeded,
}

impl SetupStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "READY",
            Self::Unreachable => "UNREACHABLE",
            Self::SetupNeeded => "SETUP_NEEDED",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resolved {
    pub database: DatabaseOverride,
    pub status: SetupStatus,
}

pub fn resolve(config: &AppConfig) -> Result<Resolved, String> {
    let store = DatabaseRegistryStore::new(&config.home);
    let registry = store.read()?;
    let (database, status) = match &registry {
        Some(registry) if reachable(registry.active()) => {
            let active = registry.active().expect("reachable implies present");
            (DatabaseOverride::File(data_file_in(Path::new(&active.path))), SetupStatus::Ready)
        }
        None if legacy_data_folder_exists(config) => {
            let adopted = adopt_legacy_folder(config);
            store.write(&adopted)?;
            let active = adopted.active().expect("adopted with itself active");
            (DatabaseOverride::File(data_file_in(Path::new(&active.path))), SetupStatus::Ready)
        }
        Some(_) => (DatabaseOverride::Memory, SetupStatus::Unreachable),
        None => (DatabaseOverride::Memory, SetupStatus::SetupNeeded),
    };
    Ok(Resolved { database: config.database.clone().unwrap_or(database), status })
}

/// The SQLite file of a registered folder.
pub fn data_file_in(folder: &Path) -> PathBuf {
    folder.join(DATA_FILE_NAME)
}

fn reachable(entry: Option<&DatabaseEntry>) -> bool {
    entry.is_some_and(|entry| Path::new(&entry.path).is_dir())
}

fn legacy_folder(config: &AppConfig) -> PathBuf {
    config.working_dir.join("data")
}

fn legacy_data_folder_exists(config: &AppConfig) -> bool {
    let folder = legacy_folder(config);
    folder.join(LEGACY_DATA_FILE_NAME).is_file() || folder.join(DATA_FILE_NAME).is_file()
}

fn adopt_legacy_folder(config: &AppConfig) -> DatabaseRegistry {
    let id = Id::random().to_string();
    let path = absolute_normalised(&legacy_folder(config)).to_string_lossy().into_owned();
    let now = Instant::now().to_string();
    DatabaseRegistry {
        active_id: Some(id.clone()),
        databases: vec![DatabaseEntry { id, label: "Local".into(), path, added_at: now.clone(), last_used_at: now }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Properties;

    fn config(home: &Path, working_dir: &Path) -> AppConfig {
        let mut config = AppConfig::from_sources(&Properties::of(&[("rekall.home", home.to_str().unwrap())]));
        config.working_dir = working_dir.to_path_buf();
        config
    }

    #[test]
    fn nothing_registered_and_no_legacy_folder_is_the_setup_screen_on_memory() {
        let home = tempfile::tempdir().unwrap();
        let work = tempfile::tempdir().unwrap();
        let resolved = resolve(&config(home.path(), work.path())).unwrap();
        assert_eq!(resolved, Resolved { database: DatabaseOverride::Memory, status: SetupStatus::SetupNeeded });
    }

    #[test]
    fn a_legacy_data_folder_is_adopted_as_local_and_opened() {
        let home = tempfile::tempdir().unwrap();
        let work = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(work.path().join("data")).unwrap();
        std::fs::File::create(work.path().join("data/rekall.mv.db")).unwrap();

        let resolved = resolve(&config(home.path(), work.path())).unwrap();
        assert_eq!(resolved.status, SetupStatus::Ready);
        assert_eq!(resolved.database, DatabaseOverride::File(work.path().join("data/rekall.db")));
        let registry = DatabaseRegistryStore::new(home.path()).read().unwrap().unwrap();
        assert_eq!(registry.active().unwrap().label, "Local");
    }

    #[test]
    fn an_active_folder_that_is_gone_is_unreachable_and_runs_on_memory() {
        let home = tempfile::tempdir().unwrap();
        let work = tempfile::tempdir().unwrap();
        let entry = DatabaseEntry {
            id: "a".into(),
            label: "Drive".into(),
            path: "/definitely/not/here".into(),
            added_at: "t".into(),
            last_used_at: "t".into(),
        };
        DatabaseRegistryStore::new(home.path())
            .write(&DatabaseRegistry { active_id: Some("a".into()), databases: vec![entry] })
            .unwrap();
        let resolved = resolve(&config(home.path(), work.path())).unwrap();
        assert_eq!(resolved, Resolved { database: DatabaseOverride::Memory, status: SetupStatus::Unreachable });
    }
}
