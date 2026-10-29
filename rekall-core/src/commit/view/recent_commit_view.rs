use rekall_kernel::Instant;
use serde::Serialize;

use crate::commit::git::LogEntry;

/// One commit of a project's recent log, as the console lists it when picking one by hand.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentCommitView {
    pub hash: String,
    pub subject: String,
    pub committed_at: Instant,
}

impl RecentCommitView {
    pub(in crate::commit) fn of(entry: LogEntry) -> Self {
        Self { hash: entry.hash, subject: entry.subject, committed_at: entry.committed_at }
    }
}
