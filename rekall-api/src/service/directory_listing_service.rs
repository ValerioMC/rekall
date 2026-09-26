//! Lists one folder on this machine for the path picker in the markdown editor.
//!
//! It reads names only, never contents, and it lives in rekall-api so it stays out of the MCP
//! tools' reach: browsing the disk is something the console does for the person typing, not
//! something a session is handed. The server is loopback-only (the local-access guard), so the
//! reach is the reach of the person already sitting at the machine.
//!
//! A path is resolved the way a shell would: `~` is the home folder, an absolute path is itself,
//! and anything else is taken relative to the folder the picker is standing in. That is what
//! lets someone type `src/comp` or `~/Downl` into the picker's filter and land in the right place.

use std::io;
use std::path::{Component, Path, PathBuf};

use rekall_common::{jstr, RekallError, Result};
use tracing::{info, warn};

use crate::dto::{DirectoryEntryResponse, DirectoryListingResponse, PathSegmentResponse};

/// Enough for any folder a person browses by eye; a folder bigger than this (a cache, a
/// node_modules) is cut off after sorting and flagged; a folder past the cut is still reached by
/// typing its path.
pub const ENTRY_LIMIT: usize = 2000;

#[derive(Clone)]
pub struct DirectoryListingService {
    home: PathBuf,
}

impl DirectoryListingService {
    pub fn new(home: impl Into<PathBuf>) -> Self {
        Self { home: normalize(&absolute(&home.into())) }
    }

    pub fn list(&self, base: Option<&str>, path: Option<&str>) -> Result<DirectoryListingResponse> {
        let folder = self.resolve(base, path)?;
        if !folder.exists() {
            return Err(RekallError::not_found_msg(format!("There is no folder at {}.", folder.display())));
        }
        if !folder.is_dir() {
            return Err(RekallError::illegal(format!("{} is a file, not a folder.", folder.display())));
        }
        let (mut entries, readable) = read_entries(&folder);
        entries.sort_by(|a, b| b.directory.cmp(&a.directory).then_with(|| jstr::compare_ignore_case(&a.name, &b.name)));
        let truncated = entries.len() > ENTRY_LIMIT;
        entries.truncate(ENTRY_LIMIT);
        Ok(DirectoryListingResponse {
            path: text(&folder),
            parent: folder.parent().map(text),
            home: text(&self.home),
            segments: segments(&folder),
            entries,
            readable,
            truncated,
        })
    }

    fn resolve(&self, base: Option<&str>, path: Option<&str>) -> Result<PathBuf> {
        let start = match base.filter(|b| !jstr::is_blank(b)) {
            None => self.home.clone(),
            Some(base) => self.expand_home(jstr::trim(base))?,
        };
        let Some(path) = path.filter(|p| !jstr::is_blank(p)) else {
            return Ok(normalize(&absolute(&start)));
        };
        let typed = self.expand_home(jstr::trim(path))?;
        Ok(normalize(&absolute(&start.join(typed))))
    }

    fn expand_home(&self, raw: &str) -> Result<PathBuf> {
        if raw.contains('\0') {
            return Err(RekallError::illegal(format!("\"{raw}\" is not a path this machine understands.")));
        }
        if raw == "~" {
            return Ok(self.home.clone());
        }
        if let Some(rest) = raw.strip_prefix("~/").or_else(|| raw.strip_prefix("~\\")) {
            return Ok(self.home.join(rest));
        }
        Ok(PathBuf::from(raw))
    }
}

/// `Path.toAbsolutePath()`: a relative path is taken against the working directory.
fn absolute(path: &Path) -> PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf())
}

/// `Path.normalize()`: `.` goes, and `name/..` cancels out, without touching the disk. A `..`
/// with nothing left above it at the root is dropped.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => match out.components().next_back() {
                Some(Component::Normal(_)) => {
                    out.pop();
                }
                Some(Component::RootDir | Component::Prefix(_)) => {}
                _ => out.push(".."),
            },
            other => out.push(other),
        }
    }
    out
}

/// The folder's children, and whether it could be read at all. A folder that exists but will
/// not be listed (a macOS privacy-protected one, say) comes back empty and unreadable.
fn read_entries(folder: &Path) -> (Vec<DirectoryEntryResponse>, bool) {
    let listed = std::fs::read_dir(folder).and_then(|children| {
        children
            .map(|child| child.map(|child| entry(&child.path(), &child.file_name().to_string_lossy())))
            .collect::<io::Result<Vec<_>>>()
    });
    match listed {
        Ok(entries) => (entries, true),
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            info!("Folder listing refused: {}", folder.display());
            (Vec::new(), false)
        }
        Err(e) => {
            warn!("Folder listing failed: {}: {e}", folder.display());
            (Vec::new(), false)
        }
    }
}

fn entry(child: &Path, name: &str) -> DirectoryEntryResponse {
    DirectoryEntryResponse {
        name: name.to_string(),
        path: text(child),
        directory: child.is_dir(),
        hidden: is_hidden(child, name),
    }
}

fn is_hidden(child: &Path, name: &str) -> bool {
    if name.starts_with('.') {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
        if let Ok(metadata) = std::fs::symlink_metadata(child) {
            return metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0;
        }
    }
    let _ = child;
    false
}

/// The breadcrumb: the filesystem root, then each folder down to this one, each with its path.
fn segments(folder: &Path) -> Vec<PathSegmentResponse> {
    let mut segments = Vec::new();
    let mut walked = PathBuf::new();
    for component in folder.components() {
        match component {
            Component::Prefix(_) => walked.push(component),
            Component::RootDir => {
                walked.push(component);
                segments.push(PathSegmentResponse { name: text(&walked), path: text(&walked) });
            }
            Component::Normal(name) => {
                walked.push(name);
                segments.push(PathSegmentResponse { name: name.to_string_lossy().into_owned(), path: text(&walked) });
            }
            Component::CurDir | Component::ParentDir => {}
        }
    }
    segments
}

fn text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
#[path = "directory_listing_service_tests.rs"]
mod tests;
