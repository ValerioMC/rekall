//! The git repositories under a project's folder. A repository is a folder holding `.git`; the walk
//! stops there, so a repository nested in another (a submodule, a vendored clone) is not listed.

use std::path::{Path, PathBuf};

use tracing::warn;

/// How far below the project's folder the walk looks: far enough for a folder of service groups.
const MAX_DEPTH: usize = 4;
const SKIPPED: [&str; 1] = ["node_modules"];

/// Every repository under `root`, `root` itself included, in path order.
pub(in crate::connection) fn repositories_under(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect(root, 0, &mut found);
    found
}

fn collect(folder: &Path, depth: usize, found: &mut Vec<PathBuf>) {
    if folder.join(".git").exists() {
        found.push(folder.to_path_buf());
        return;
    }
    if depth == MAX_DEPTH {
        return;
    }
    for child in subfolders(folder) {
        collect(&child, depth + 1, found);
    }
}

/// The folders to walk into, by name: hidden ones, dependency caches and symbolic links are passed
/// over. An unreadable folder is logged and walked no further.
fn subfolders(folder: &Path) -> Vec<PathBuf> {
    let entries = match std::fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(failure) => {
            warn!(%failure, folder = %folder.display(), "A folder could not be read while looking for repositories");
            return Vec::new();
        }
    };
    let mut children: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .filter(|entry| is_walked(&entry.file_name().to_string_lossy()))
        .map(|entry| entry.path())
        .collect();
    children.sort();
    children
}

fn is_walked(name: &str) -> bool {
    !name.starts_with('.') && !SKIPPED.contains(&name)
}

#[cfg(test)]
#[path = "../../../tests/unit/connection/support/repository_folders_tests.rs"]
mod tests;
