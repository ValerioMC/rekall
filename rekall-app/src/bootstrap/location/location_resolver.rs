use std::path::{Path, PathBuf};

use rekall_common::{Id, Instant};
use rekall_repository::DATA_FILE_NAME;

use super::super::folder::{absolute_normalised, LEGACY_DATA_FILE_NAME};
use super::super::registry::{DatabaseEntry, DatabaseRegistry, DatabaseRegistryStore};
use crate::config::{AppConfig, DatabaseOverride};

use super::{Resolved, SetupStatus};

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
#[path = "../../../test/bootstrap/location/location_resolver_tests.rs"]
mod tests;
