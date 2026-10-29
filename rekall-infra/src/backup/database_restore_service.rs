use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use futures::StreamExt;
use rekall_core::backup::{BackupFile, BackupReason, DatabaseRestore, UploadChunks};
use rekall_kernel::{InternalFailure, RekallError, Result};
use tracing::{error, info, warn};

use super::{DatabaseBackupService, DatabaseLocation};
use crate::restart::Restarter;

/// The first bytes of every SQLite database file.
const SQLITE_HEADER: &[u8; 16] = b"SQLite format 3\0";
const STAGED_SUFFIX: &str = ".restoring";

#[derive(Clone)]
pub struct DatabaseRestoreService {
    backups: DatabaseBackupService,
    restarter: Restarter,
}

impl DatabaseRestoreService {
    pub fn new(backups: DatabaseBackupService, restarter: Restarter) -> Self {
        Self { backups, restarter }
    }

    /// Restores one of the backups in the folder, by the name it is listed under.
    pub async fn restore(&self, name: &str) -> Result<BackupFile> {
        let zip = self.backups.resolve(name)?;
        self.restore_from(&zip).await
    }

    /// Keeps an uploaded backup in the folder as `…-uploaded.zip`, then restores it.
    async fn restore_file(&self, upload: &Path) -> Result<BackupFile> {
        let location = self.require_location()?;
        let failed = || RekallError::internal(InternalFailure::Io, "Could not keep the uploaded backup");
        std::fs::create_dir_all(location.backups()).map_err(|_| failed())?;
        let kept = self.backups.fresh_name(&location.backups(), BackupReason::Uploaded);
        std::fs::copy(upload, &kept).map_err(|_| failed())?;
        match self.restore_from(&kept).await {
            Ok(safety) => Ok(safety),
            Err(refused) => {
                delete_quietly(&kept);
                Err(refused)
            }
        }
    }

    async fn restore_from(&self, zip: &Path) -> Result<BackupFile> {
        let location = self.require_location()?;
        let staged = location.folder.join(format!("{}{STAGED_SUFFIX}", location.data_file_name));
        extract_database(zip, &location.data_file_name, &staged)?;
        if let Err(refused) = rekall_store::check_openable(&staged).await {
            delete_quietly(&staged);
            return Err(RekallError::illegal(format!("{refused} Nothing was restored.")));
        }
        if !self.restarter.can_restart() {
            delete_quietly(&staged);
            return Err(RekallError::conflict("This instance cannot restart itself, so nothing was restored."));
        }

        let safety = match self.backups.back_up(BackupReason::BeforeRestore).await {
            Ok(safety) => safety,
            Err(failed) => {
                delete_quietly(&staged);
                return Err(failed);
            }
        };
        let live = location.data_file();
        let swap_staged = staged.clone();
        let scheduled = self.restarter.restart_with(Box::new(|| Ok(())), Box::new(move || swap(&swap_staged, &live)));
        if !scheduled {
            delete_quietly(&staged);
            return Err(RekallError::conflict("This instance cannot restart itself, so nothing was restored."));
        }
        info!(
            backup = %zip.file_name().unwrap_or_default().to_string_lossy(),
            live = %location.data_file().display(),
            safety = %safety.name,
            "Restoring a backup; the previous state is kept in the safety backup"
        );
        Ok(safety)
    }

    fn require_location(&self) -> Result<DatabaseLocation> {
        self.backups
            .location()
            .cloned()
            .ok_or_else(|| RekallError::conflict("This database is not a file on disk, so it cannot be restored."))
    }
}

#[async_trait]
impl DatabaseRestore for DatabaseRestoreService {
    async fn restore(&self, name: &str) -> Result<BackupFile> {
        DatabaseRestoreService::restore(self, name).await
    }

    async fn restore_upload(&self, upload: UploadChunks<'_>) -> Result<BackupFile> {
        let staged = staged(upload).await?;
        self.restore_file(staged.path()).await
    }
}

/// The upload written to a temporary file, deleted when it is dropped.
async fn staged(mut upload: UploadChunks<'_>) -> Result<tempfile::NamedTempFile> {
    let unreadable = || RekallError::internal(InternalFailure::Io, "Could not read the uploaded file");
    let mut temp = tempfile::NamedTempFile::new().map_err(|_| unreadable())?;
    let mut size = 0u64;
    while let Some(chunk) = upload.next().await {
        let chunk = chunk?;
        size += chunk.len() as u64;
        temp.write_all(&chunk).map_err(|_| unreadable())?;
    }
    temp.flush().map_err(|_| unreadable())?;
    if size == 0 {
        return Err(RekallError::illegal("The uploaded file is empty. Nothing was restored."));
    }
    Ok(temp)
}

/// Writes the database file out of the zip, refusing a zip that does not hold one. A file with
/// the database's own name wins; otherwise the zip has to hold exactly one `.db`, which is what
/// lets a backup taken from a database with another name be restored here.
pub fn extract_database(zip: &Path, data_file_name: &str, target: &Path) -> Result<()> {
    let unreadable = || RekallError::illegal("That file is not a zip Rekall can read. Nothing was restored.");
    let entry_name = database_entry_in(zip, data_file_name)?;
    let file = std::fs::File::open(zip).map_err(|_| unreadable())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|_| unreadable())?;
    let written = (|| -> std::io::Result<()> {
        let mut entry = archive.by_name(&entry_name).map_err(std::io::Error::other)?;
        let mut out = std::fs::File::create(target)?;
        std::io::copy(&mut entry, &mut out)?;
        Ok(())
    })();
    if written.is_err() {
        delete_quietly(target);
        return Err(unreadable());
    }
    if !starts_with_sqlite_header(target) {
        delete_quietly(target);
        return Err(RekallError::illegal(format!(
            "'{entry_name}' in that zip is not a SQLite database file. Nothing was restored."
        )));
    }
    Ok(())
}

fn database_entry_in(zip: &Path, data_file_name: &str) -> Result<String> {
    let unreadable = || RekallError::illegal("That file is not a zip Rekall can read. Nothing was restored.");
    let file = std::fs::File::open(zip).map_err(|_| unreadable())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|_| unreadable())?;
    let mut single = None;
    let mut databases = 0;
    for index in 0..archive.len() {
        let entry = archive.by_index(index).map_err(|_| unreadable())?;
        let name = entry.name().to_string();
        if entry.is_dir() || name.contains('/') || name.contains('\\') {
            continue;
        }
        if !name.ends_with(".db") {
            continue;
        }
        if name == data_file_name {
            return Ok(name);
        }
        databases += 1;
        single = Some(name);
    }
    match (databases, single) {
        (1, Some(name)) => Ok(name),
        (0, _) => Err(RekallError::illegal("That zip holds no Rekall database file (*.db). Nothing was restored.")),
        _ => Err(RekallError::illegal("That zip holds more than one database file. Nothing was restored.")),
    }
}

fn starts_with_sqlite_header(file: &Path) -> bool {
    let mut head = [0u8; 16];
    std::fs::File::open(file).and_then(|mut f| f.read_exact(&mut head)).is_ok() && &head == SQLITE_HEADER
}

/// Runs with no instance up and the pool closed: drops the old write-ahead log, which belongs to
/// the file being replaced, and moves the staged file in. If the move fails the old file is still
/// in place and the application comes back on it, with the failure in the log.
fn swap(staged: &Path, live: &Path) -> std::result::Result<(), String> {
    for suffix in ["-wal", "-shm"] {
        let companion = PathBuf::from(format!("{}{suffix}", live.display()));
        if companion.exists() {
            if let Err(failed) = std::fs::remove_file(&companion) {
                warn!(%failed, file = %companion.display(), "Could not delete a database companion file");
            }
        }
    }
    match std::fs::rename(staged, live) {
        Ok(()) => {
            info!(live = %live.display(), "Restored the database file");
            Ok(())
        }
        Err(failed) => {
            error!(%failed, live = %live.display(), "The restore could not replace the database; the application restarts on the file it had");
            delete_quietly(staged);
            Err(failed.to_string())
        }
    }
}

pub fn delete_quietly(file: &Path) {
    if let Err(failed) = std::fs::remove_file(file) {
        if failed.kind() != std::io::ErrorKind::NotFound {
            warn!(%failed, file = %file.display(), "Could not delete a staged file");
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/backup/database_restore_service_tests.rs"]
mod tests;
