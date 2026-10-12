use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::transcript::{flatten, folder_name};

const SCAN_BYTES: u64 = 256 * 1024;
const SESSIONS_TRIED: usize = 3;

/// The folder a Claude Code project folder stands for. Its name cannot be read back as a path, so
/// a session transcript inside names it, or else the disk is searched for a folder that flattens
/// to the name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkingDirectory {
    pub path: String,
    /// False when neither a transcript nor the disk named it: the path is a guess from the name,
    /// and no folder by that name is there.
    pub resolved: bool,
}

#[derive(Deserialize)]
struct CwdLine {
    cwd: Option<String>,
}

impl WorkingDirectory {
    pub fn recover(project_folder: &Path, folder: &str) -> Self {
        match recorded_in_sessions(project_folder, folder).or_else(|| found_on_disk(folder)) {
            Some(path) => Self { path, resolved: true },
            None => Self { path: folder.replace('-', "/"), resolved: false },
        }
    }
}

/// A recorded `cwd` counts only when it flattens to the folder's own name: a session that moved
/// elsewhere reports where it ended up, not where it started.
fn recorded_in_sessions(project_folder: &Path, folder: &str) -> Option<String> {
    let mut sessions: Vec<_> = std::fs::read_dir(project_folder)
        .ok()?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|extension| extension == "jsonl"))
        .collect();
    sessions.sort();
    for session in sessions.iter().take(SESSIONS_TRIED) {
        let Some(head) = head_of(session) else { continue };
        if let Some(cwd) = head.lines().filter_map(cwd_of).find(|cwd| folder_name(Path::new(cwd)) == folder) {
            return Some(cwd);
        }
    }
    None
}

/// Walks down from the root through entries whose flattened name starts what is left of the
/// folder's name, abandoning a branch that dead-ends.
fn found_on_disk(folder: &str) -> Option<String> {
    let remaining = folder.strip_prefix('-')?;
    descend(Path::new("/"), remaining).map(|path| path.to_string_lossy().into_owned())
}

fn descend(directory: &Path, remaining: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(directory).ok()?.filter_map(Result::ok) {
        let flattened = flatten(&entry.file_name().to_string_lossy());
        if remaining == flattened {
            return Some(entry.path());
        }
        let Some(rest) = remaining.strip_prefix(&flattened).and_then(|rest| rest.strip_prefix('-')) else { continue };
        if let Some(found) = descend(&entry.path(), rest) {
            return Some(found);
        }
    }
    None
}

fn head_of(session: &Path) -> Option<String> {
    let mut bytes = Vec::new();
    File::open(session).ok()?.take(SCAN_BYTES).read_to_end(&mut bytes).ok()?;
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

fn cwd_of(line: &str) -> Option<String> {
    if !line.contains("\"cwd\"") {
        return None;
    }
    serde_json::from_str::<CwdLine>(line).ok()?.cwd
}

#[cfg(test)]
#[path = "../../tests/unit/resources/working_directory_tests.rs"]
mod tests;
