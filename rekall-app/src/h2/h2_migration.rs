use std::path::{Path, PathBuf};

use rekall_common::{Id, Instant};

use crate::bootstrap::registry::{DatabaseEntry, DatabaseRegistry, DatabaseRegistryStore};
use crate::{bootstrap, AppConfig};

use super::{ImportReport, Importer};

/// `--migrate-from-h2 <path>`: import the Java build's database (a folder holding
/// `rekall.mv.db`, or the file itself) into `rekall.db` beside it, then make that folder the
/// active database, so the server that starts next opens it.
pub async fn migrate_from_h2(config: &AppConfig, source: &Path, h2_jar: Option<PathBuf>) -> Result<ImportReport, String> {
    let h2_file = if source.is_dir() { source.join(bootstrap::folder::LEGACY_DATA_FILE_NAME) } else { source.to_path_buf() };
    let h2_file = bootstrap::folder::absolute_normalised(&h2_file);
    let folder = h2_file.parent().ok_or("That path has no folder")?.to_path_buf();
    let name = h2_file.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let stem = name.strip_suffix(".mv.db").ok_or_else(|| format!("{} is not an H2 file (*.mv.db)", h2_file.display()))?;
    let target = folder.join(format!("{stem}.db"));

    let importer = Importer::locate_with(h2_jar)?;
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
