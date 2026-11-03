/// A file that was already modified when the snapshot was taken, and what it held then. A file
/// already deleted has no hash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotEntry {
    pub path: String,
    pub content_hash: Option<String>,
}

/// Every file a working tree held modified, new or deleted at one moment, each with a hash of
/// its content, so a later reading can tell which of them a session has touched since.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkTreeSnapshot {
    pub entries: Vec<SnapshotEntry>,
}
