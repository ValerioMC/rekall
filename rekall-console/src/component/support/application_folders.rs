//! The applications a repository holds. A repository whose root has no manifest and whose folders
//! hold two or more is a group of applications (`backend/` with one service per folder), each its own
//! component; any other repository is one application.

use std::path::{Path, PathBuf};

use super::repository_folders::subfolders;

/// Far enough below the repository's root for a folder of service groups.
const MAX_DEPTH: usize = 3;
const MANIFESTS: [&str; 8] = ["Cargo.toml", "package.json", "pyproject.toml", "pom.xml", "build.gradle", "build.gradle.kts", "go.mod", "composer.json"];
const SKIPPED: [&str; 3] = ["target", "vendor", "dist"];

/// `repository` itself, or every application folder inside it in path order.
pub(crate) fn applications_in(repository: &Path) -> Vec<PathBuf> {
    if holds_manifest(repository) {
        return vec![repository.to_path_buf()];
    }
    let mut found = Vec::new();
    collect(repository, 0, &mut found);
    if found.len() < 2 {
        return vec![repository.to_path_buf()];
    }
    found
}

fn collect(folder: &Path, depth: usize, found: &mut Vec<PathBuf>) {
    if depth > 0 && holds_manifest(folder) {
        found.push(folder.to_path_buf());
        return;
    }
    if depth == MAX_DEPTH {
        return;
    }
    for child in subfolders(folder).into_iter().filter(|child| !is_skipped(child)) {
        collect(&child, depth + 1, found);
    }
}

fn holds_manifest(folder: &Path) -> bool {
    MANIFESTS.iter().any(|name| folder.join(name).is_file())
}

fn is_skipped(folder: &Path) -> bool {
    folder.file_name().is_some_and(|name| SKIPPED.contains(&name.to_string_lossy().as_ref()))
}

#[cfg(test)]
#[path = "../../../tests/unit/component/support/application_folders_tests.rs"]
mod tests;
