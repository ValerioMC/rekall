use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use futures::StreamExt;
use rekall_core::backup::{BackupFile, BackupReason, DatabaseRestore};
use rekall_core::Clock;
use rekall_kernel::Instant;
use rekall_store::Database;
use tempfile::TempDir;
use tokio::sync::mpsc::UnboundedReceiver;

use super::*;
use crate::backup::{DatabaseBackupService, DatabaseLocation, DatabaseRestoreService};
use crate::restart::{RestartRequest, Restarter};

const PICTURE: &str = "acme/web/web-12/login-flow-3f9a1c2b.png";

/// One database in a folder of its own, with a restarter whose requests the test receives.
struct Installation {
    folder: TempDir,
    backups: DatabaseBackupService,
    restore: DatabaseRestoreService,
    _restarts: UnboundedReceiver<RestartRequest>,
    _database: Database,
}

impl Installation {
    fn image(&self, relative: &str) -> PathBuf {
        self.folder.path().join("images").join(relative)
    }

    fn keep_picture(&self, relative: &str, bytes: &[u8]) {
        let file = self.image(relative);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, bytes).unwrap();
    }

    async fn back_up(&self) -> BackupFile {
        self.backups.back_up(BackupReason::Manual).await.unwrap()
    }

    fn zip_of(&self, backup: &BackupFile) -> PathBuf {
        self.backups.resolve(&backup.name).unwrap()
    }

    async fn restore_zip(&self, zip: &Path) -> Result<BackupFile> {
        let bytes = std::fs::read(zip).unwrap();
        self.restore.restore_upload(futures::stream::iter(vec![Ok(bytes)]).boxed()).await
    }
}

/// A second later at every reading, so each backup has a newer name than the one before.
fn ticking_clock() -> Clock {
    let start = Instant::parse("2026-11-26T12:00:00Z").unwrap();
    let ticks = AtomicI64::new(0);
    Arc::new(move || start.plus_seconds(ticks.fetch_add(1, Ordering::SeqCst)))
}

async fn installation_keeping(keep: usize) -> Installation {
    let folder = tempfile::tempdir().unwrap();
    let data_file = folder.path().join("rekall.db");
    let database = rekall_store::open(&data_file).await.unwrap();
    let backups = DatabaseBackupService::new(database.conn.clone(), DatabaseLocation::of(&data_file), false, 24, keep, ticking_clock());
    let (restarter, restarts) = Restarter::channel();
    let restore = DatabaseRestoreService::new(backups.clone(), restarter);
    Installation { folder, backups, restore, _restarts: restarts, _database: database }
}

async fn installation() -> Installation {
    installation_keeping(10).await
}

fn entries(zip: &Path) -> Vec<(String, CompressionMethod)> {
    let mut archive = ZipArchive::new(File::open(zip).unwrap()).unwrap();
    (0..archive.len())
        .map(|index| {
            let entry = archive.by_index(index).unwrap();
            (entry.name().to_string(), entry.compression())
        })
        .collect()
}

fn zip_with(folder: &Path, held: &[(&str, &[u8])]) -> PathBuf {
    let path = folder.join("handmade.zip");
    let mut writer = ZipWriter::new(File::create(&path).unwrap());
    for (name, bytes) in held {
        writer.start_file(*name, SimpleFileOptions::default()).unwrap();
        writer.write_all(bytes).unwrap();
    }
    writer.finish().unwrap();
    path
}

async fn database_bytes() -> Vec<u8> {
    let source = installation().await;
    let backup = source.back_up().await;
    let mut archive = ZipArchive::new(File::open(source.zip_of(&backup)).unwrap()).unwrap();
    let mut bytes = Vec::new();
    archive.by_name("rekall.db").unwrap().read_to_end(&mut bytes).unwrap();
    bytes
}

#[tokio::test]
async fn a_backup_holds_every_picture_uncompressed_beside_the_database() {
    let installation = installation().await;
    installation.keep_picture(PICTURE, b"pixels");
    installation.keep_picture("acme/web/web-12/notes.txt", b"not a picture");

    let backup = installation.back_up().await;

    assert_eq!(
        entries(&installation.zip_of(&backup)),
        [("rekall.db".to_string(), CompressionMethod::Deflated), (format!("images/{PICTURE}"), CompressionMethod::Stored)]
    );
}

#[tokio::test]
async fn without_pictures_a_backup_holds_the_database_alone() {
    let installation = installation().await;

    let backup = installation.back_up().await;

    assert_eq!(entries(&installation.zip_of(&backup)), [("rekall.db".to_string(), CompressionMethod::Deflated)]);
}

#[tokio::test]
async fn restoring_on_an_empty_folder_brings_the_pictures_back() {
    let source = installation().await;
    source.keep_picture(PICTURE, b"pixels");
    let backup = source.back_up().await;
    let elsewhere = installation().await;

    elsewhere.restore_zip(&source.zip_of(&backup)).await.unwrap();

    assert_eq!(std::fs::read(elsewhere.image(PICTURE)).unwrap(), b"pixels");
}

#[tokio::test]
async fn restoring_writes_over_a_changed_picture_and_keeps_one_the_zip_does_not_hold() {
    let installation = installation().await;
    installation.keep_picture(PICTURE, b"pixels");
    let backup = installation.back_up().await;
    installation.keep_picture(PICTURE, b"damaged");
    installation.keep_picture("acme/web/web-12/newer-00000000.png", b"newer");

    installation.restore.restore(&backup.name).await.unwrap();

    assert_eq!(std::fs::read(installation.image(PICTURE)).unwrap(), b"pixels");
    assert_eq!(std::fs::read(installation.image("acme/web/web-12/newer-00000000.png")).unwrap(), b"newer");
}

#[tokio::test]
async fn the_oldest_backup_still_restores_its_pictures_when_the_safety_backup_prunes_it() {
    let installation = installation_keeping(1).await;
    installation.keep_picture(PICTURE, b"pixels");
    let backup = installation.back_up().await;
    std::fs::remove_file(installation.image(PICTURE)).unwrap();

    installation.restore.restore(&backup.name).await.unwrap();

    assert!(installation.backups.resolve(&backup.name).is_err());
    assert_eq!(std::fs::read(installation.image(PICTURE)).unwrap(), b"pixels");
}

#[tokio::test]
async fn an_entry_that_is_not_a_picture_path_is_ignored() {
    let installation = installation().await;
    let handmade = tempfile::tempdir().unwrap();
    let database = database_bytes().await;
    let zip = zip_with(
        handmade.path(),
        &[("rekall.db", &database), ("images/../x.png", b"escape"), ("images/acme/web/web-12/script.sh", b"run"), ("x.png", b"loose")],
    );

    installation.restore_zip(&zip).await.unwrap();

    assert!(!installation.folder.path().join("x.png").exists());
    assert!(!installation.folder.path().join("images").exists());
}

#[tokio::test]
async fn a_zip_whose_database_is_refused_writes_no_picture() {
    let installation = installation().await;
    let handmade = tempfile::tempdir().unwrap();
    let zip = zip_with(handmade.path(), &[("rekall.db", b"SQLite format 3\0 and nothing more"), (&format!("images/{PICTURE}"), b"pixels")]);

    installation.restore_zip(&zip).await.unwrap_err();

    assert!(!installation.image(PICTURE).exists());
}
