use std::path::PathBuf;

/// Where the database comes from when it is set by hand (`rekall.database.url`, `REKALL_DB_URL`)
/// instead of by the registry in `~/.rekall/config.json`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DatabaseOverride {
    /// A SQLite file.
    File(PathBuf),
    /// A database that lives as long as the process.
    Memory,
}

impl DatabaseOverride {
    /// `sqlite::memory:` (or `:memory:`) is in memory; `sqlite:<path>`, `sqlite://<path>` and a
    /// bare path are that file, with any `?options` dropped.
    pub fn parse(url: &str) -> Option<Self> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }
        if url == "sqlite::memory:" || url == ":memory:" {
            return Some(Self::Memory);
        }
        let path = url.strip_prefix("sqlite://").or_else(|| url.strip_prefix("sqlite:")).unwrap_or(url);
        let path = path.split('?').next().unwrap_or(path);
        Some(Self::File(PathBuf::from(path)))
    }
}
