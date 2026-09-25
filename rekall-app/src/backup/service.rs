//! `DatabaseBackupService`.
//!
//! - A backup is taken when the application is ready and then checked every hour: one is due
//!   when the newest backup of any kind is older than `rekall.backup.interval-hours`.
//! - `back_up` takes one on demand, and a restore takes one first.
//! - Only the newest `rekall.backup.keep` are kept, whatever their reason.
//! - A database with no file (in memory) has no backups; nothing here runs on one.
//!
//! A failure is logged and reported by `status()`; it never stops the application.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use chrono::{NaiveDateTime, TimeDelta};
use regex::Regex;
use rekall_common::{Instant, RekallError, Result};
use rekall_service::Clock;
use sea_orm::{ConnectionTrait, DatabaseConnection};
use serde::Serialize;
use tokio::task::JoinHandle;
use tracing::{info, warn};

use crate::bootstrap::folder::absolute_normalised;

/// The only names this folder hands out or accepts back, which is what keeps a name from being a path.
pub static NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^rekall-(\d{8}-\d{6})(?:-(\d{1,3}))?-(auto|manual|before-restore|uploaded)\.zip$").unwrap()
});

const STAMP: &str = "%Y%m%d-%H%M%S";
const CHECK_EVERY: Duration = Duration::from_secs(3600);
const FIRST_CHECK_AFTER: Duration = Duration::from_secs(5);

/// Why a backup was taken; it is the last part of the file's name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BackupReason {
    Auto,
    Manual,
    BeforeRestore,
    Uploaded,
}

impl BackupReason {
    pub fn slug(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Manual => "manual",
            Self::BeforeRestore => "before-restore",
            Self::Uploaded => "uploaded",
        }
    }

    fn of_slug(slug: &str) -> Option<Self> {
        [Self::Auto, Self::Manual, Self::BeforeRestore, Self::Uploaded].into_iter().find(|r| r.slug() == slug)
    }
}

/// One backup in the database folder's `backups/`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupFile {
    pub name: String,
    pub size_bytes: u64,
    pub created_at: Instant,
    pub reason: BackupReason,
}

/// What the settings panel shows: where backups go, which there are, and the last thing that failed.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupStatus {
    pub available: bool,
    pub folder: Option<String>,
    pub interval_hours: i64,
    pub keep: usize,
    pub backups: Vec<BackupFile>,
    pub last_failure: Option<String>,
}

/// Where the open database lives on disk: the folder that holds it and its file's name. Only a
/// file database has one; an in-memory one, which the setup screen runs on, has none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatabaseLocation {
    pub folder: PathBuf,
    pub data_file_name: String,
}

impl DatabaseLocation {
    pub fn of(data_file: &Path) -> Option<Self> {
        let absolute = absolute_normalised(data_file);
        let folder = absolute.parent()?.to_path_buf();
        let data_file_name = absolute.file_name()?.to_string_lossy().into_owned();
        Some(Self { folder, data_file_name })
    }

    pub fn backups(&self) -> PathBuf {
        self.folder.join("backups")
    }

    pub fn data_file(&self) -> PathBuf {
        self.folder.join(&self.data_file_name)
    }
}

struct Inner {
    conn: DatabaseConnection,
    location: Option<DatabaseLocation>,
    enabled: bool,
    interval: TimeDelta,
    keep: usize,
    clock: Clock,
    writing: tokio::sync::Mutex<()>,
    last_failure: Mutex<Option<String>>,
    scheduler: Mutex<Option<JoinHandle<()>>>,
}

#[derive(Clone)]
pub struct DatabaseBackupService {
    inner: Arc<Inner>,
}

impl DatabaseBackupService {
    pub fn new(
        conn: DatabaseConnection,
        location: Option<DatabaseLocation>,
        enabled: bool,
        interval_hours: i64,
        keep: usize,
        clock: Clock,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                conn,
                location,
                enabled,
                interval: TimeDelta::hours(interval_hours),
                keep: keep.max(1),
                clock,
                writing: tokio::sync::Mutex::new(()),
                last_failure: Mutex::new(None),
                scheduler: Mutex::new(None),
            }),
        }
    }

    pub fn status(&self) -> Result<BackupStatus> {
        let inner = &self.inner;
        Ok(BackupStatus {
            available: inner.location.is_some(),
            folder: inner.location.as_ref().map(|l| l.backups().to_string_lossy().into_owned()),
            interval_hours: inner.interval.num_hours(),
            keep: inner.keep,
            backups: if inner.location.is_some() { self.list()? } else { Vec::new() },
            last_failure: inner.last_failure.lock().expect("never poisoned").clone(),
        })
    }

    pub fn location(&self) -> Option<&DatabaseLocation> {
        self.inner.location.as_ref()
    }

    /// Takes a backup now and prunes the oldest past `keep`.
    pub async fn back_up(&self, reason: BackupReason) -> Result<BackupFile> {
        let _writing = self.inner.writing.lock().await;
        let Some(found) = self.inner.location.clone() else {
            return Err(RekallError::conflict("This database is not a file on disk, so it has no backups."));
        };
        let io_failure = || RekallError::internal("UncheckedIOException", format!("Could not write to {}", found.backups().display()));
        std::fs::create_dir_all(found.backups()).map_err(|_| self.failed(io_failure()))?;
        let target = self.fresh_name(&found.backups(), reason);
        let copy = found.backups().join(format!(".{}.copy", target.file_name().unwrap().to_string_lossy()));
        let _ = std::fs::remove_file(&copy);
        let sql = format!("VACUUM INTO '{}'", copy.to_string_lossy().replace('\'', "''"));
        if let Err(refused) = self.inner.conn.execute_unprepared(&sql).await {
            let _ = std::fs::remove_file(&copy);
            return Err(self.failed(RekallError::internal(
                "IllegalStateException",
                format!("SQLite refused the backup: {refused}"),
            )));
        }
        let zipped = zip_single(&copy, &found.data_file_name, &target);
        let _ = std::fs::remove_file(&copy);
        if zipped.is_err() {
            let _ = std::fs::remove_file(&target);
            return Err(self.failed(io_failure()));
        }
        self.prune(&found.backups())?;
        *self.inner.last_failure.lock().expect("never poisoned") = None;
        info!("Backed up the database to {}", target.display());
        describe(&target).ok_or_else(|| io_failure())
    }

    /// Newest first.
    pub fn list(&self) -> Result<Vec<BackupFile>> {
        let Some(folder) = self.inner.location.as_ref().map(DatabaseLocation::backups) else {
            return Ok(Vec::new());
        };
        if !folder.is_dir() {
            return Ok(Vec::new());
        }
        let entries = std::fs::read_dir(&folder)
            .map_err(|_| RekallError::internal("UncheckedIOException", format!("Could not read {}", folder.display())))?;
        let mut files: Vec<BackupFile> = entries.filter_map(|e| e.ok()).filter_map(|e| describe(&e.path())).collect();
        files.sort_by(|a, b| b.name.cmp(&a.name));
        Ok(files)
    }

    /// The file behind a name this service handed out; anything else is refused.
    pub fn resolve(&self, name: &str) -> Result<PathBuf> {
        if !NAME.is_match(name) {
            return Err(RekallError::illegal(format!("'{name}' is not the name of a backup")));
        }
        let Some(found) = &self.inner.location else {
            return Err(RekallError::conflict("This database is not a file on disk, so it has no backups."));
        };
        let file = found.backups().join(name);
        if !file.is_file() {
            return Err(RekallError::illegal(format!("There is no backup called '{name}'")));
        }
        Ok(file)
    }

    /// A name for a file about to be written into the backups folder, with the reason in it.
    pub fn fresh_name(&self, folder: &Path, reason: BackupReason) -> PathBuf {
        let stamp = (self.inner.clock)().datetime().format(STAMP).to_string();
        let mut candidate = folder.join(format!("rekall-{stamp}-{}.zip", reason.slug()));
        let mut attempt = 1;
        while candidate.exists() {
            candidate = folder.join(format!("rekall-{stamp}-{attempt}-{}.zip", reason.slug()));
            attempt += 1;
        }
        candidate
    }

    pub async fn back_up_if_due(&self) {
        if !self.inner.enabled || self.inner.location.is_none() {
            return;
        }
        let cutoff = (self.inner.clock)().minus(self.inner.interval);
        let due = match self.list() {
            Ok(list) => list.first().map(|newest| newest.created_at.is_before(&cutoff)).unwrap_or(true),
            Err(failed) => {
                warn!("The scheduled backup failed: {}", failed.message());
                return;
            }
        };
        if !due {
            return;
        }
        if let Err(failed) = self.back_up(BackupReason::Auto).await {
            warn!("The scheduled backup failed: {}", failed.message());
        }
    }

    /// `ApplicationReadyEvent`: check five seconds in, then every hour.
    pub fn start_schedule(&self) {
        if !self.inner.enabled || self.inner.location.is_none() {
            return;
        }
        let service = self.clone();
        let handle = tokio::spawn(async move {
            tokio::time::sleep(FIRST_CHECK_AFTER).await;
            loop {
                service.back_up_if_due().await;
                tokio::time::sleep(CHECK_EVERY).await;
            }
        });
        *self.inner.scheduler.lock().expect("never poisoned") = Some(handle);
    }

    /// `ContextClosedEvent`.
    pub fn stop_schedule(&self) {
        if let Some(handle) = self.inner.scheduler.lock().expect("never poisoned").take() {
            handle.abort();
        }
    }

    fn prune(&self, folder: &Path) -> Result<()> {
        let all = self.list()?;
        for old in all.iter().skip(self.inner.keep) {
            if let Err(failed) = std::fs::remove_file(folder.join(&old.name)) {
                if failed.kind() != std::io::ErrorKind::NotFound {
                    warn!("Could not delete the old backup {}: {failed}", old.name);
                }
            }
        }
        Ok(())
    }

    fn failed(&self, failure: RekallError) -> RekallError {
        let message = failure.message().to_string();
        let message = message.split_once(": ").map(|(_, m)| m.to_string()).unwrap_or(message);
        warn!("Backup failed: {message}");
        *self.inner.last_failure.lock().expect("never poisoned") = Some(message);
        failure
    }
}

fn zip_single(source: &Path, entry_name: &str, target: &Path) -> std::io::Result<()> {
    let file = std::fs::File::create(target)?;
    let mut writer = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    writer.start_file(entry_name, options).map_err(std::io::Error::other)?;
    let mut input = std::fs::File::open(source)?;
    std::io::copy(&mut input, &mut writer)?;
    writer.finish().map_err(std::io::Error::other)?.flush()
}

pub fn describe(file: &Path) -> Option<BackupFile> {
    let name = file.file_name()?.to_string_lossy().into_owned();
    let captures = NAME.captures(&name)?;
    if !file.is_file() {
        return None;
    }
    let created = NaiveDateTime::parse_from_str(&captures[1], STAMP).ok()?.and_utc();
    let size_bytes = match file.metadata() {
        Ok(metadata) => metadata.len(),
        Err(failed) => {
            warn!("Skipping the backup {}, which could not be read: {failed}", file.display());
            return None;
        }
    };
    Some(BackupFile {
        name: name.clone(),
        size_bytes,
        created_at: Instant::from_datetime(created),
        reason: BackupReason::of_slug(&captures[3])?,
    })
}
