//! `FolderValidation`: what the setup screen needs to know about a folder a person typed, before
//! anything is registered.

use std::path::{Component, Path, PathBuf};

use rekall_repository::DATA_FILE_NAME;

/// The file the Java build kept its H2 database in. A folder holding one has a database too: the
/// first start on it imports it (see `crate::h2`).
pub const LEGACY_DATA_FILE_NAME: &str = "rekall.mv.db";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inspection {
    pub resolved_path: String,
    pub exists: bool,
    pub is_directory: bool,
    pub writable: bool,
    pub has_database: bool,
}

impl Inspection {
    pub fn usable(&self) -> bool {
        self.exists && self.is_directory && self.writable
    }
}

pub fn inspect(raw_path: Option<&str>, user_home: &Path) -> Inspection {
    let trimmed = raw_path.map(str::trim).unwrap_or("");
    if trimmed.is_empty() {
        return Inspection { resolved_path: String::new(), exists: false, is_directory: false, writable: false, has_database: false };
    }
    let path = expand(trimmed, user_home);
    let exists = path.exists() || path.symlink_metadata().is_ok();
    let is_directory = exists && path.is_dir();
    let writable = is_directory && is_writable(&path);
    let has_database = is_directory && (path.join(DATA_FILE_NAME).is_file() || path.join(LEGACY_DATA_FILE_NAME).is_file());
    Inspection { resolved_path: path.to_string_lossy().into_owned(), exists, is_directory, writable, has_database }
}

/// `~` and `~/...` are the user's home; the rest is made absolute against the working directory
/// and normalised lexically, as `Path.toAbsolutePath().normalize()` did.
fn expand(trimmed: &str, user_home: &Path) -> PathBuf {
    let with_home = if trimmed == "~" || trimmed.starts_with("~/") {
        format!("{}{}", user_home.to_string_lossy(), &trimmed[1..])
    } else {
        trimmed.to_string()
    };
    absolute_normalised(Path::new(&with_home))
}

pub fn absolute_normalised(path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(path)
    };
    let mut normalised = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalised.pop();
            }
            other => normalised.push(other.as_os_str()),
        }
    }
    normalised
}

#[cfg(unix)]
pub fn is_writable(path: &Path) -> bool {
    use std::os::unix::ffi::OsStrExt;
    let Ok(c_path) = std::ffi::CString::new(path.as_os_str().as_bytes()) else { return false };
    // SAFETY: a valid NUL-terminated path; access(2) only reads it.
    unsafe { libc::access(c_path.as_ptr(), libc::W_OK) == 0 }
}

#[cfg(not(unix))]
pub fn is_writable(path: &Path) -> bool {
    path.metadata().map(|m| !m.permissions().readonly()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn home() -> PathBuf {
        PathBuf::from("/home/someone")
    }

    #[test]
    fn a_folder_that_does_not_exist_is_reported_as_such() {
        let temp = tempfile::tempdir().unwrap();
        let result = inspect(Some(temp.path().join("nowhere").to_str().unwrap()), &home());
        assert!(!result.exists);
        assert!(!result.usable());
    }

    #[test]
    fn an_empty_existing_folder_is_usable_but_has_no_database_yet() {
        let temp = tempfile::tempdir().unwrap();
        let result = inspect(temp.path().to_str(), &home());
        assert!(result.usable());
        assert!(!result.has_database);
    }

    #[test]
    fn a_folder_with_an_mv_db_file_or_a_sqlite_one_is_reported_as_already_having_a_database() {
        let legacy = tempfile::tempdir().unwrap();
        std::fs::File::create(legacy.path().join("rekall.mv.db")).unwrap();
        assert!(inspect(legacy.path().to_str(), &home()).has_database);

        let current = tempfile::tempdir().unwrap();
        std::fs::File::create(current.path().join("rekall.db")).unwrap();
        assert!(inspect(current.path().to_str(), &home()).has_database);
    }

    #[test]
    fn blank_input_is_never_usable() {
        assert!(!inspect(Some("   "), &home()).usable());
        assert!(!inspect(None, &home()).usable());
    }

    #[test]
    fn a_file_rather_than_a_folder_is_not_usable() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("not-a-folder");
        std::fs::File::create(&file).unwrap();
        let result = inspect(file.to_str(), &home());
        assert!(result.exists);
        assert!(!result.is_directory);
        assert!(!result.usable());
    }

    #[test]
    fn a_leading_tilde_expands_to_the_home_directory() {
        assert_eq!(inspect(Some("~"), &home()).resolved_path, "/home/someone");
        assert_eq!(inspect(Some("~/a/../b"), &home()).resolved_path, "/home/someone/b");
    }
}
