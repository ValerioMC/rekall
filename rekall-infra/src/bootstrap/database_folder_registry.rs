use std::path::PathBuf;

use rekall_core::settings::{DatabaseFolders, DatabaseRegistry, FolderInspection};
use rekall_kernel::{InternalFailure, RekallError, Result};

use super::folder::inspect;
use super::registry::DatabaseRegistryStore;

/// `DatabaseFolders` over `config.json` and the local disk.
pub struct DatabaseFolderRegistry {
    store: DatabaseRegistryStore,
    user_home: PathBuf,
}

impl DatabaseFolderRegistry {
    pub fn new(store: DatabaseRegistryStore, user_home: PathBuf) -> Self {
        Self { store, user_home }
    }
}

impl DatabaseFolders for DatabaseFolderRegistry {
    fn read(&self) -> Result<Option<DatabaseRegistry>> {
        self.store.read().map_err(|failed| RekallError::internal(InternalFailure::Io, failed))
    }

    fn write(&self, registry: &DatabaseRegistry) -> Result<()> {
        self.store.write(registry).map_err(|failed| RekallError::internal(InternalFailure::Io, failed))
    }

    fn inspect(&self, path: Option<&str>) -> FolderInspection {
        inspect(path, &self.user_home)
    }
}
