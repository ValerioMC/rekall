use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::SessionFiles;

const TRANSCRIPT_EXTENSION: &str = "jsonl";

/// Claude Code's transcript folder: one folder per working directory, named after its path, with
/// a `<session>.jsonl` per session and that session's subagents under `<session>/subagents/`.
#[derive(Clone, Debug)]
pub struct TranscriptFolder {
    root: PathBuf,
}

impl TranscriptFolder {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Every session started in `project_folder` or in a folder under it. A root that is not
    /// there yet holds no sessions.
    pub fn sessions_of(&self, project_folder: &Path) -> io::Result<Vec<SessionFiles>> {
        let name = folder_name(project_folder);
        let mut sessions = Vec::new();
        let entries = match fs::read_dir(&self.root) {
            Ok(entries) => entries,
            Err(missing) if missing.kind() == io::ErrorKind::NotFound => return Ok(sessions),
            Err(other) => return Err(other),
        };
        for entry in entries {
            let entry = entry?;
            let folder = entry.file_name().to_string_lossy().into_owned();
            if entry.file_type()?.is_dir() && is_at_or_under(&folder, &name) {
                sessions.extend(sessions_in(&entry.path())?);
            }
        }
        sessions.sort_by(|a, b| a.session_id.cmp(&b.session_id));
        Ok(sessions)
    }
}

/// The name Claude Code gives a working directory's folder: every character but a letter or a
/// digit becomes `-`.
fn folder_name(project_folder: &Path) -> String {
    let path = project_folder.to_string_lossy();
    path.trim_end_matches(['/', '\\']).chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect()
}

/// The project's folder or one under it. A sibling such as `vega-old` passes too, which costs a
/// read and counts nothing: tokens follow anchors, not folders.
fn is_at_or_under(folder: &str, project: &str) -> bool {
    folder == project || folder.strip_prefix(project).is_some_and(|rest| rest.starts_with('-'))
}

fn sessions_in(folder: &Path) -> io::Result<Vec<SessionFiles>> {
    let mut sessions = Vec::new();
    for entry in fs::read_dir(folder)? {
        let path = entry?.path();
        if !is_transcript(&path) {
            continue;
        }
        let Some(session_id) = path.file_stem().map(|stem| stem.to_string_lossy().into_owned()) else { continue };
        let subagents = subagents_of(&folder.join(&session_id))?;
        let mut bytes = fs::metadata(&path)?.len();
        for subagent in &subagents {
            bytes += fs::metadata(subagent)?.len();
        }
        sessions.push(SessionFiles { session_id, main: path, subagents, bytes });
    }
    Ok(sessions)
}

fn subagents_of(session_folder: &Path) -> io::Result<Vec<PathBuf>> {
    let folder = session_folder.join("subagents");
    let entries = match fs::read_dir(&folder) {
        Ok(entries) => entries,
        Err(missing) if missing.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(other) => return Err(other),
    };
    let mut subagents = Vec::new();
    for entry in entries {
        let path = entry?.path();
        if is_transcript(&path) {
            subagents.push(path);
        }
    }
    subagents.sort();
    Ok(subagents)
}

fn is_transcript(path: &Path) -> bool {
    path.is_file() && path.extension().is_some_and(|extension| extension == TRANSCRIPT_EXTENSION)
}

#[cfg(test)]
#[path = "../../tests/unit/transcript/transcript_folder_tests.rs"]
mod tests;
