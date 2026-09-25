//! `DatabaseRestoreService`: puts a backup back as the database, which is also how a database
//! moves to another folder or another machine: download a backup there, restore it here.
//!
//! A restore replaces everything, so nothing is touched until three things hold: the zip holds a
//! SQLite database file, a backup of what is there now has been taken (so the restore can itself
//! be undone), and the file has been extracted beside the live one. Only then does the application
//! restart, and in the gap between the old instance closing and the new one starting the file is
//! swapped in. The new instance runs the migrations as always, so a backup from an older version
//! comes back migrated.

use std::io::Read;
use std::path::{Path, PathBuf};

use rekall_common::{RekallError, Result};
use tracing::{error, info, warn};

use super::service::{BackupFile, BackupReason, DatabaseBackupService, DatabaseLocation};
use crate::restart::Restarter;

/// The first bytes of every SQLite database file.
const SQLITE_HEADER: &[u8; 16] = b"SQLite format 3\0";
const STAGED_SUFFIX: &str = ".restoring";
const H2_SUFFIX: &str = ".mv.db";

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
    pub async fn restore_upload(&self, upload: &Path) -> Result<BackupFile> {
        let location = self.require_location()?;
        let failed = || RekallError::internal("UncheckedIOException", "Could not keep the uploaded backup");
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
            "Restoring {} over {}; the previous state is in {}",
            zip.file_name().unwrap_or_default().to_string_lossy(),
            location.data_file().display(),
            safety.name
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
    let mut h2 = false;
    for index in 0..archive.len() {
        let entry = archive.by_index(index).map_err(|_| unreadable())?;
        let name = entry.name().to_string();
        if entry.is_dir() || name.contains('/') || name.contains('\\') {
            continue;
        }
        if name.ends_with(H2_SUFFIX) {
            h2 = true;
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
        (0, _) if h2 => Err(RekallError::illegal(
            "That zip holds an H2 database from the Java build of Rekall. Import it with \
             `rekall-server --migrate-from-h2 <folder>`. Nothing was restored.",
        )),
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
                warn!("Could not delete {}: {failed}", companion.display());
            }
        }
    }
    match std::fs::rename(staged, live) {
        Ok(()) => {
            info!("Restored the database file {}", live.display());
            Ok(())
        }
        Err(failed) => {
            error!("The restore could not replace {}; the application restarts on the file it had: {failed}", live.display());
            delete_quietly(staged);
            Err(failed.to_string())
        }
    }
}

pub fn delete_quietly(file: &Path) {
    if let Err(failed) = std::fs::remove_file(file) {
        if failed.kind() != std::io::ErrorKind::NotFound {
            warn!("Could not delete {}: {failed}", file.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn zip(folder: &Path, entries: &[(&str, &[u8])]) -> PathBuf {
        let path = folder.join(format!("backup-{}.zip", rekall_common::Id::random()));
        let mut writer = zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
        for (name, content) in entries {
            writer.start_file(*name, zip::write::SimpleFileOptions::default()).unwrap();
            writer.write_all(content).unwrap();
        }
        writer.finish().unwrap();
        path
    }

    fn sqlite(rest: &str) -> Vec<u8> {
        let mut bytes = SQLITE_HEADER.to_vec();
        bytes.extend_from_slice(rest.as_bytes());
        bytes
    }

    #[test]
    fn the_database_file_is_taken_out_of_the_zip_under_its_own_name() {
        let folder = tempfile::tempdir().unwrap();
        let zip = zip(folder.path(), &[("rekall.db", &sqlite("the rest"))]);
        let target = folder.path().join("rekall.db.restoring");
        extract_database(&zip, "rekall.db", &target).unwrap();
        assert!(std::fs::read(&target).unwrap().starts_with(b"SQLite format 3"));
    }

    #[test]
    fn a_backup_of_a_database_with_another_name_is_accepted_when_it_is_the_only_one_in_the_zip() {
        let folder = tempfile::tempdir().unwrap();
        let zip = zip(folder.path(), &[("other.db", &sqlite("data"))]);
        let target = folder.path().join("rekall.db.restoring");
        extract_database(&zip, "rekall.db", &target).unwrap();
        assert!(target.exists());
    }

    #[test]
    fn a_zip_with_no_database_two_databases_or_a_file_that_is_not_sqlite_is_refused_and_leaves_nothing_behind() {
        let folder = tempfile::tempdir().unwrap();
        let target = folder.path().join("rekall.db.restoring");
        let refused = |entries: &[(&str, &[u8])]| {
            extract_database(&zip(folder.path(), entries), "rekall.db", &target).unwrap_err().message().to_string()
        };
        assert!(refused(&[("notes.md", b"hello")]).contains("no Rekall database file"));
        assert!(refused(&[("a.db", &sqlite("")), ("b.db", &sqlite(""))]).contains("more than one"));
        assert!(refused(&[("rekall.db", b"PK not a database")]).contains("not a SQLite database file"));
        assert!(refused(&[("rekall.mv.db", b"H:2 data")]).contains("--migrate-from-h2"));
        assert!(!target.exists());
    }

    #[test]
    fn a_file_that_is_not_a_zip_at_all_is_refused() {
        let folder = tempfile::tempdir().unwrap();
        let not_zip = folder.path().join("x.zip");
        std::fs::write(&not_zip, "plain text").unwrap();
        let refused = extract_database(&not_zip, "rekall.db", &folder.path().join("out")).unwrap_err();
        assert!(matches!(refused, RekallError::IllegalArgument(_)));
    }

    #[test]
    fn only_a_file_database_has_a_location_its_folder_its_file_and_a_backups_folder_beside_it() {
        let location = DatabaseLocation::of(Path::new("/data/rekall/rekall.db")).unwrap();
        assert_eq!(location.folder, PathBuf::from("/data/rekall"));
        assert_eq!(location.data_file(), PathBuf::from("/data/rekall/rekall.db"));
        assert_eq!(location.backups(), PathBuf::from("/data/rekall/backups"));
    }
}
