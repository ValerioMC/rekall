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
mod tests {
    use super::*;

    struct Fixture {
        home: tempfile::TempDir,
        service: DirectoryListingService,
    }

    impl Fixture {
        fn new() -> Self {
            let home = tempfile::tempdir().unwrap();
            let project = home.path().join("project");
            std::fs::create_dir_all(project.join("src/components")).unwrap();
            std::fs::create_dir_all(project.join("Docs")).unwrap();
            std::fs::write(project.join("README.md"), "#").unwrap();
            std::fs::write(project.join("app.ts"), "").unwrap();
            std::fs::write(project.join(".env.local"), "").unwrap();
            let service = DirectoryListingService::new(home.path());
            Self { home, service }
        }

        fn at(&self, relative: &str) -> PathBuf {
            let home = normalize(&absolute(self.home.path()));
            if relative.is_empty() {
                home
            } else {
                home.join(relative)
            }
        }

        fn path(&self, relative: &str) -> String {
            text(&self.at(relative))
        }
    }

    #[test]
    fn folders_come_first_then_files_each_by_name_ignoring_case_hidden_ones_flagged() {
        let fixture = Fixture::new();
        let listing = fixture.service.list(Some(&fixture.path("project")), None).unwrap();

        let names: Vec<&str> = listing.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["Docs", "src", ".env.local", "app.ts", "README.md"]);
        let hidden: Vec<&str> = listing.entries.iter().filter(|e| e.hidden).map(|e| e.name.as_str()).collect();
        assert_eq!(hidden, [".env.local"]);
        assert_eq!(listing.entries[0].path, fixture.path("project/Docs"));
        assert!(listing.readable);
        assert!(!listing.truncated);
    }

    #[test]
    fn a_typed_path_is_taken_relative_to_the_folder_the_picker_stands_in_and_normalised() {
        let fixture = Fixture::new();
        let base = fixture.path("project");

        assert_eq!(
            fixture.service.list(Some(&base), Some("src/components/../components")).unwrap().path,
            fixture.path("project/src/components")
        );
        assert_eq!(fixture.service.list(Some(&base), Some("..")).unwrap().path, fixture.path(""));
    }

    #[test]
    fn tilde_is_the_home_folder_and_an_absolute_path_ignores_the_base() {
        let fixture = Fixture::new();
        let base = fixture.path("project/src");

        assert_eq!(fixture.service.list(Some(&base), Some("~")).unwrap().path, fixture.path(""));
        assert_eq!(fixture.service.list(Some(&base), Some("~/project")).unwrap().path, fixture.path("project"));
        assert_eq!(
            fixture.service.list(Some(&base), Some(&fixture.path("project/Docs"))).unwrap().path,
            fixture.path("project/Docs")
        );
    }

    #[test]
    fn with_no_base_and_no_path_the_listing_is_the_home_folder() {
        let fixture = Fixture::new();
        let listing = fixture.service.list(None, Some("  ")).unwrap();

        assert_eq!(listing.path, fixture.path(""));
        assert_eq!(listing.home, fixture.path(""));
    }

    #[test]
    fn the_breadcrumb_runs_from_the_root_to_the_folder_each_crumb_carrying_its_own_path() {
        let fixture = Fixture::new();
        let folder = fixture.at("project/src");

        let listing = fixture.service.list(Some(&text(&folder)), None).unwrap();

        let root: PathBuf = folder.ancestors().last().unwrap().to_path_buf();
        assert_eq!(listing.segments.first().unwrap().path, text(&root));
        assert_eq!(listing.segments.last().unwrap(), &PathSegmentResponse { name: "src".into(), path: text(&folder) });
        assert_eq!(listing.parent, Some(fixture.path("project")));
    }

    #[cfg(unix)]
    #[test]
    fn the_root_has_no_parent_and_a_dot_dot_above_it_stays_there() {
        assert_eq!(normalize(Path::new("/..")), PathBuf::from("/"));
        assert_eq!(normalize(Path::new("/a/./b/../c")), PathBuf::from("/a/c"));
        let listing = DirectoryListingService::new("/").list(Some("/"), None).unwrap();
        assert_eq!(listing.parent, None);
        assert_eq!(listing.segments, [PathSegmentResponse { name: "/".into(), path: "/".into() }]);
    }

    #[test]
    fn a_missing_folder_is_not_found_and_a_file_is_refused_as_a_folder() {
        let fixture = Fixture::new();
        let base = fixture.path("project");

        assert!(matches!(fixture.service.list(Some(&base), Some("nope")), Err(RekallError::NotFound(_))));
        match fixture.service.list(Some(&base), Some("README.md")) {
            Err(RekallError::IllegalArgument(message)) => assert!(message.contains("is a file")),
            other => panic!("expected a refusal, got {other:?}"),
        }
    }

    #[test]
    fn a_folder_bigger_than_the_limit_is_cut_after_sorting_and_flagged() {
        let fixture = Fixture::new();
        let crowded = fixture.at("crowded");
        std::fs::create_dir_all(&crowded).unwrap();
        for index in 0..=ENTRY_LIMIT {
            std::fs::write(crowded.join(format!("file-{index:05}")), "").unwrap();
        }

        let listing = fixture.service.list(Some(&text(&crowded)), None).unwrap();

        assert!(listing.truncated);
        assert_eq!(listing.entries.len(), ENTRY_LIMIT);
        assert_eq!(listing.entries.last().unwrap().name, format!("file-{:05}", ENTRY_LIMIT - 1));
    }
}
