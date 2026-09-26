use std::path::{Path, PathBuf};

use super::DatabaseRegistry;

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
#[path = "database_registry_store_tests.rs"]
mod tests;
