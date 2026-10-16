use std::path::{Path, PathBuf};

use rekall_common::{Id, Instant};
use rekall_repository::DATA_FILE_NAME;

use super::super::folder::absolute_normalised;
use super::super::registry::{DatabaseEntry, DatabaseRegistry, DatabaseRegistryStore};
use crate::config::DatabaseOverride;

use super::{Resolved, SetupStatus};

/// Where the database is: the registry under `home` decides, `database` (an explicit override)
/// wins over it, and `working_dir/data` is adopted when no registry exists yet.
pub fn resolve(home: &Path, database: Option<&DatabaseOverride>, working_dir: &Path) -> Result<Resolved, String> {
    let store = DatabaseRegistryStore::new(home);
    let registry = store.read()?;
    let (resolved_database, status) = match &registry {
        Some(registry) if reachable(registry.active()) => {
            let active = registry.active().expect("reachable implies present");
            (DatabaseOverride::File(data_file_in(Path::new(&active.path))), SetupStatus::Ready)
        }
        None if data_folder_beside_the_app_has_a_database(working_dir) => {
            let adopted = adopt_data_folder_beside_the_app(working_dir);
            store.write(&adopted)?;
            let active = adopted.active().expect("adopted with itself active");
            (DatabaseOverride::File(data_file_in(Path::new(&active.path))), SetupStatus::Ready)
        }
        Some(_) => (DatabaseOverride::Memory, SetupStatus::Unreachable),
        None => (DatabaseOverride::Memory, SetupStatus::SetupNeeded),
    };
    Ok(Resolved { database: database.cloned().unwrap_or(resolved_database), status })
}

/// The SQLite file of a registered folder.
pub fn data_file_in(folder: &Path) -> PathBuf {
    folder.join(DATA_FILE_NAME)
}

fn reachable(entry: Option<&DatabaseEntry>) -> bool {
    entry.is_some_and(|entry| Path::new(&entry.path).is_dir())
}

/// `./data` under the working directory: where a database lived before folders were registered.
fn data_folder_beside_the_app(working_dir: &Path) -> PathBuf {
    working_dir.join("data")
}

fn data_folder_beside_the_app_has_a_database(working_dir: &Path) -> bool {
    data_file_in(&data_folder_beside_the_app(working_dir)).is_file()
}

/// With no registry yet, a database already in `./data` becomes the registered, active one.
fn adopt_data_folder_beside_the_app(working_dir: &Path) -> DatabaseRegistry {
    let id = Id::random().to_string();
    let path = absolute_normalised(&data_folder_beside_the_app(working_dir)).to_string_lossy().into_owned();
    let now = Instant::now().to_string();
    DatabaseRegistry {
        active_id: Some(id.clone()),
        databases: vec![DatabaseEntry { id, label: "Local".into(), path, added_at: now.clone(), last_used_at: now }],
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/bootstrap/location/location_resolver_tests.rs"]
mod tests;
