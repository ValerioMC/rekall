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
#[path = "folder_tests.rs"]
mod tests;
