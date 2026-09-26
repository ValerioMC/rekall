use std::path::PathBuf;

/// Where the database comes from when it is set by hand (`REKALL_DB_URL`,
/// `spring.datasource.url`) instead of by the registry in `~/.rekall/config.json`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DatabaseOverride {
    /// A SQLite file.
    File(PathBuf),
    /// A database that lives as long as the process.
    Memory,
}

impl DatabaseOverride {
    /// Accepts the JDBC URLs the Java build took, so a script that set one keeps working:
    /// `jdbc:h2:file:<base>[;options]` is the SQLite file `<base>.db` beside where H2 kept
    /// `<base>.mv.db`, and `jdbc:h2:mem:...` is in memory. `sqlite:<path>`, `sqlite::memory:` and a
    /// bare path are read as themselves.
    pub fn parse(url: &str) -> Option<Self> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }
        if url.starts_with("jdbc:h2:mem:") || url == "sqlite::memory:" || url == ":memory:" {
            return Some(Self::Memory);
        }
        if let Some(rest) = url.strip_prefix("jdbc:h2:file:") {
            let base = rest.split(';').next().unwrap_or(rest);
            return Some(Self::File(PathBuf::from(format!("{base}.db"))));
        }
        let path = url.strip_prefix("sqlite://").or_else(|| url.strip_prefix("sqlite:")).unwrap_or(url);
        let path = path.split('?').next().unwrap_or(path);
        Some(Self::File(PathBuf::from(path)))
    }
}
