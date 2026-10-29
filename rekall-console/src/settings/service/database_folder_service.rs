use std::path::Path;
use std::sync::Arc;

use rekall_core::settings::{ApplicationRestart, DatabaseEntry, DatabaseFolders, DatabaseRegistry, FolderInspection, SetupStatus};
use rekall_kernel::{Id, Instant, RekallError, Result};

use crate::settings::{AddDatabaseCommand, AddedDatabaseView, DatabaseFoldersView, DatabaseView};

#[derive(Clone)]
pub struct DatabaseFolderService {
    folders: Arc<dyn DatabaseFolders>,
    restart: Arc<dyn ApplicationRestart>,
}

impl DatabaseFolderService {
    pub fn new(folders: Arc<dyn DatabaseFolders>, restart: Arc<dyn ApplicationRestart>) -> Self {
        Self { folders, restart }
    }

    pub fn status(&self) -> Result<DatabaseFoldersView> {
        let Some(registry) = self.folders.read()? else {
            return Ok(DatabaseFoldersView { status: SetupStatus::SetupNeeded, active: None, databases: Vec::new() });
        };
        let databases: Vec<DatabaseView> = registry.databases.iter().map(|entry| self.view(entry, &registry)).collect();
        let active = databases.iter().find(|view| view.active).cloned();
        Ok(DatabaseFoldersView { status: setup_status(active.as_ref()), active, databases })
    }

    /// Registers nothing: what the setup screen shows while a person types a path.
    pub fn check(&self, path: &str) -> FolderInspection {
        self.folders.inspect(Some(path))
    }

    /// Registers the folder as the active database and restarts onto it.
    pub fn add(&self, command: AddDatabaseCommand) -> Result<AddedDatabaseView> {
        let check = self.folders.inspect(command.path.as_deref());
        refuse_unusable(&check)?;
        let current = self.folders.read()?.unwrap_or_else(DatabaseRegistry::empty);
        let now = Instant::now().to_string();
        let entry = DatabaseEntry {
            id: Id::random().to_string(),
            label: label_or_folder_name(command.label.as_deref(), &check.resolved_path),
            path: check.resolved_path.clone(),
            added_at: now.clone(),
            last_used_at: now,
        };
        let mut databases = current.databases;
        databases.push(entry.clone());
        let updated = DatabaseRegistry { active_id: Some(entry.id.clone()), databases };
        self.folders.write(&updated)?;
        self.restart.restart();
        Ok(AddedDatabaseView { opened_existing: check.has_database, entry: self.view(&entry, &updated) })
    }

    /// Makes a registered folder the active database and restarts onto it.
    pub fn activate(&self, id: &str) -> Result<DatabaseView> {
        let current = self.require_registry()?;
        let entry = find(&current, id)?;
        if !self.reachable(entry) {
            return Err(RekallError::illegal(format!("\"{}\" points at {}, which is not reachable right now.", entry.label, entry.path)));
        }
        let used = DatabaseEntry { last_used_at: Instant::now().to_string(), ..entry.clone() };
        let registry = DatabaseRegistry { active_id: Some(id.to_string()), databases: with_entry(&current, used) };
        self.folders.write(&registry)?;
        self.restart.restart();
        Ok(self.view(find(&registry, id)?, &registry))
    }

    pub fn rename(&self, id: &str, label: Option<&str>) -> Result<DatabaseView> {
        let Some(label) = label.map(str::trim).filter(|label| !label.is_empty()) else {
            return Err(RekallError::illegal("A database needs a name."));
        };
        let current = self.require_registry()?;
        let renamed = DatabaseEntry { label: label.to_string(), ..find(&current, id)?.clone() };
        let registry = DatabaseRegistry { active_id: current.active_id.clone(), databases: with_entry(&current, renamed) };
        self.folders.write(&registry)?;
        Ok(self.view(find(&registry, id)?, &registry))
    }

    /// Drops the folder from the registry and leaves its files where they are.
    pub fn forget(&self, id: &str) -> Result<()> {
        let current = self.require_registry()?;
        find(&current, id)?;
        if current.active_id.as_deref() == Some(id) {
            return Err(RekallError::conflict("This is the database currently in use. Switch to another one before forgetting it."));
        }
        let remaining = current.databases.iter().filter(|entry| entry.id != id).cloned().collect();
        self.folders.write(&DatabaseRegistry { active_id: current.active_id.clone(), databases: remaining })
    }

    fn require_registry(&self) -> Result<DatabaseRegistry> {
        self.folders.read()?.ok_or_else(|| RekallError::not_found_msg("No database is configured yet"))
    }

    fn reachable(&self, entry: &DatabaseEntry) -> bool {
        self.folders.inspect(Some(&entry.path)).is_directory
    }

    fn view(&self, entry: &DatabaseEntry, registry: &DatabaseRegistry) -> DatabaseView {
        DatabaseView {
            id: entry.id.clone(),
            label: entry.label.clone(),
            path: entry.path.clone(),
            active: registry.active_id.as_deref() == Some(entry.id.as_str()),
            reachable: self.reachable(entry),
            added_at: entry.added_at.clone(),
            last_used_at: entry.last_used_at.clone(),
        }
    }
}

fn setup_status(active: Option<&DatabaseView>) -> SetupStatus {
    match active {
        None => SetupStatus::SetupNeeded,
        Some(active) if active.reachable => SetupStatus::Ready,
        Some(_) => SetupStatus::Unreachable,
    }
}

fn refuse_unusable(check: &FolderInspection) -> Result<()> {
    if !check.exists || !check.is_directory {
        return Err(RekallError::illegal(format!("The folder \"{}\" does not exist. Create it, then try again.", check.resolved_path)));
    }
    if !check.writable {
        return Err(RekallError::illegal(format!("The folder \"{}\" is not writable.", check.resolved_path)));
    }
    Ok(())
}

fn label_or_folder_name(label: Option<&str>, resolved_path: &str) -> String {
    match label.map(str::trim) {
        Some(label) if !label.is_empty() => label.to_string(),
        _ => Path::new(resolved_path).file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_else(|| "Database".into()),
    }
}

fn find<'a>(registry: &'a DatabaseRegistry, id: &str) -> Result<&'a DatabaseEntry> {
    registry
        .databases
        .iter()
        .find(|entry| entry.id == id)
        .ok_or_else(|| RekallError::not_found_msg(format!("No database registered with id {id}")))
}

/// The registry's entries with the one sharing `changed`'s id replaced by it.
fn with_entry(registry: &DatabaseRegistry, changed: DatabaseEntry) -> Vec<DatabaseEntry> {
    registry.databases.iter().map(|candidate| if candidate.id == changed.id { changed.clone() } else { candidate.clone() }).collect()
}

#[cfg(test)]
#[path = "../../../tests/unit/settings/service/database_folder_service_tests.rs"]
mod tests;
