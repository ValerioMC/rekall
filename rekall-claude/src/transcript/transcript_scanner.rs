use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use rekall_common::{InternalFailure, RekallError, Result};
use rekall_service::token::{TokenUsageService, UnitUsage};
use tokio::time::MissedTickBehavior;
use tracing::{info, warn};

use super::{SessionFiles, SessionLedger, TranscriptFolder};

/// Reads the transcripts of every session started in a project's folder into `token_usage`,
/// skipping the ones that have not grown since they were last read. Sessions started anywhere
/// else never name a project's task often enough to be worth the read.
pub struct TranscriptScanner {
    folder: TranscriptFolder,
    tokens: TokenUsageService,
}

impl TranscriptScanner {
    pub fn new(folder: TranscriptFolder, tokens: TokenUsageService) -> Arc<Self> {
        Arc::new(Self { folder, tokens })
    }

    /// Scan now, then every `every`, until the handle is aborted.
    pub fn start(self: &Arc<Self>, every: Duration) -> tokio::task::JoinHandle<()> {
        let this = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(every.max(Duration::from_secs(1)));
            interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
            loop {
                interval.tick().await;
                match this.scan().await {
                    Ok(0) => {}
                    Ok(read) => info!(sessions = read, "Read Claude Code transcripts for token usage"),
                    Err(failed) => warn!(%failed, "Reading Claude Code transcripts failed"),
                }
            }
        })
    }

    /// Read every grown session of every watched folder, and say how many were read. A session
    /// whose folder sits under two projects' folders is read once.
    pub async fn scan(&self) -> Result<usize> {
        let mut seen = HashSet::new();
        let mut read = 0;
        for project_folder in self.tokens.watched_folders().await? {
            for session in self.sessions_of(project_folder).await? {
                if seen.insert(session.session_id.clone()) && self.read_if_grown(session).await? {
                    read += 1;
                }
            }
        }
        Ok(read)
    }

    /// A folder that cannot be listed is skipped this round and tried again the next.
    async fn sessions_of(&self, project_folder: PathBuf) -> Result<Vec<SessionFiles>> {
        let folder = self.folder.clone();
        let listed = in_background(move || folder.sessions_of(&project_folder)).await?;
        Ok(listed.unwrap_or_else(|failed| {
            warn!(%failed, "Could not list Claude Code's transcripts");
            Vec::new()
        }))
    }

    /// A transcript that cannot be read keeps its last reading, so it is tried again next round.
    async fn read_if_grown(&self, session: SessionFiles) -> Result<bool> {
        if self.tokens.scanned_bytes(&session.session_id).await? == Some(session.bytes) {
            return Ok(false);
        }
        let (session_id, bytes) = (session.session_id.clone(), session.bytes);
        let spent = match in_background(move || read_session(&session)).await? {
            Ok(spent) => spent,
            Err(failed) => {
                warn!(%failed, %session_id, "Could not read a Claude Code transcript");
                return Ok(false);
            }
        };
        self.tokens.record_session(&session_id, bytes, &spent).await?;
        Ok(true)
    }
}

fn read_session(session: &SessionFiles) -> io::Result<Vec<UnitUsage>> {
    let mut ledger = SessionLedger::new();
    for_each_line(&session.main, |line| ledger.read_line(line))?;
    for subagent in &session.subagents {
        for_each_line(subagent, |line| ledger.read_subagent_line(line))?;
    }
    Ok(ledger.spent())
}

/// Bytes that are not UTF-8 are replaced rather than stopping the read.
fn for_each_line(path: &Path, mut read: impl FnMut(&str)) -> io::Result<()> {
    for line in BufReader::new(File::open(path)?).split(b'\n') {
        read(&String::from_utf8_lossy(&line?));
    }
    Ok(())
}

async fn in_background<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> Result<T> {
    tokio::task::spawn_blocking(work).await.map_err(|failed| RekallError::internal(InternalFailure::Background, failed))
}

#[cfg(test)]
#[path = "../../tests/unit/transcript/transcript_scanner_tests.rs"]
mod tests;
