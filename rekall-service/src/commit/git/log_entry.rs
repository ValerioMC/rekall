use rekall_common::Instant;

/// One line of the recent log: enough to recognise a commit, without its patch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogEntry {
    pub hash: String,
    pub subject: String,
    pub committed_at: Instant,
}
