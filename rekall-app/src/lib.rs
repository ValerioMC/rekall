//! The Rekall server: every module composed into one Axum application on one port, with what
//! only the application itself does — which database to open (`bootstrap`), backups and restores
//! (`backup`), the local-access guard (`security`), the console's static bundle (`spa`), the
//! health endpoint (`actuator`), restarting onto another database (`restart`, `server`), and
//! importing a database from the Java build's H2 file (`h2`).
//!
//! `rekall-server` is this as a plain binary; the desktop shell (`rekall-app/desktop`) runs the
//! same `server::start` in-process and points its window at it.

pub mod actuator;
pub mod backup;
pub mod bootstrap;
pub mod config;
pub mod h2;
pub mod methods;
pub mod restart;
pub mod security;
pub mod server;
pub mod spa;

use std::path::{Path, PathBuf};

use bootstrap::registry::{DatabaseEntry, DatabaseRegistry, DatabaseRegistryStore};
pub use config::{AppConfig, Properties};
use rekall_common::{Id, Instant};
pub use server::{start, Running, StartOptions};

/// `--migrate-from-h2 <path>`: import the Java build's database (a folder holding
/// `rekall.mv.db`, or the file itself) into `rekall.db` beside it, then make that folder the
/// active database, so the server that starts next opens it.
pub async fn migrate_from_h2(config: &AppConfig, source: &Path, h2_jar: Option<PathBuf>) -> Result<h2::ImportReport, String> {
    let h2_file = if source.is_dir() { source.join(bootstrap::folder::LEGACY_DATA_FILE_NAME) } else { source.to_path_buf() };
    let h2_file = bootstrap::folder::absolute_normalised(&h2_file);
    let folder = h2_file.parent().ok_or("That path has no folder")?.to_path_buf();
    let name = h2_file.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let stem = name.strip_suffix(".mv.db").ok_or_else(|| format!("{} is not an H2 file (*.mv.db)", h2_file.display()))?;
    let target = folder.join(format!("{stem}.db"));

    let importer = h2::Importer::locate_with(h2_jar)?;
    let (from, into) = (h2_file.clone(), target.clone());
    let report = tokio::task::spawn_blocking(move || importer.import_blocking(&from, &into))
        .await
        .map_err(|e| e.to_string())??;

    if config.database.is_none() && stem == "rekall" {
        register_as_active(config, &folder)?;
    }
    Ok(report)
}

fn register_as_active(config: &AppConfig, folder: &Path) -> Result<(), String> {
    let store = DatabaseRegistryStore::new(&config.home);
    let mut registry = store.read()?.unwrap_or_else(DatabaseRegistry::empty);
    let path = folder.to_string_lossy().into_owned();
    let now = Instant::now().to_string();
    let id = match registry.databases.iter_mut().find(|entry| entry.path == path) {
        Some(entry) => {
            entry.last_used_at = now;
            entry.id.clone()
        }
        None => {
            let id = Id::random().to_string();
            let label = folder.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "Database".into());
            registry.databases.push(DatabaseEntry { id: id.clone(), label, path, added_at: now.clone(), last_used_at: now });
            id
        }
    };
    registry.active_id = Some(id);
    store.write(&registry)
}
