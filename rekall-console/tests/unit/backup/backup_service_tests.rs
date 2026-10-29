use async_trait::async_trait;
use futures::StreamExt;
use rekall_core::backup::BackupReason;
use rekall_kernel::{Instant, RekallError};

use super::*;

const KNOWN: &str = "rekall-20260101-000000-manual.zip";

fn backup(name: &str, size_bytes: u64) -> BackupFile {
    BackupFile { name: name.into(), size_bytes, created_at: Instant::now(), reason: BackupReason::BeforeRestore }
}

/// One backup, `KNOWN`, holding three bytes.
struct FakeBackups;

#[async_trait]
impl DatabaseBackups for FakeBackups {
    fn status(&self) -> Result<BackupStatus> {
        Ok(BackupStatus { available: true, folder: None, interval_hours: 24, keep: 10, backups: vec![backup(KNOWN, 3)], last_failure: None })
    }

    async fn back_up_now(&self) -> Result<BackupFile> {
        Ok(backup(KNOWN, 3))
    }

    async fn read(&self, name: &str) -> Result<Vec<u8>> {
        match name {
            KNOWN => Ok(vec![1, 2, 3]),
            _ => Err(RekallError::illegal(format!("'{name}' is not the name of a backup"))),
        }
    }
}

/// Answers with a safety backup whose size is the number of bytes the upload carried.
struct FakeRestore;

#[async_trait]
impl DatabaseRestore for FakeRestore {
    async fn restore(&self, name: &str) -> Result<BackupFile> {
        Ok(backup(name, 0))
    }

    async fn restore_upload(&self, upload: UploadChunks<'_>) -> Result<BackupFile> {
        let chunks: Vec<Result<Vec<u8>>> = upload.collect().await;
        let size = chunks.into_iter().map(|chunk| chunk.map(|bytes| bytes.len() as u64)).sum::<Result<u64>>()?;
        Ok(backup("safety", size))
    }
}

fn service() -> BackupService {
    BackupService::new(Arc::new(FakeBackups), Arc::new(FakeRestore))
}

#[tokio::test]
async fn a_listed_backup_downloads_and_any_other_name_is_refused() {
    let service = service();

    assert_eq!(service.status().unwrap().backups[0].name, KNOWN);
    assert_eq!(service.download(KNOWN).await.unwrap(), vec![1, 2, 3]);
    assert!(matches!(service.download("../rekall.db").await, Err(RekallError::IllegalArgument(_))));
}

#[tokio::test]
async fn an_upload_reaches_the_restore_chunk_by_chunk() {
    let upload = futures::stream::iter(vec![Ok(vec![0; 4]), Ok(vec![0; 6])]).boxed();

    let safety = service().restore_upload(upload).await.unwrap();

    assert_eq!(safety.size_bytes, 10);
}
