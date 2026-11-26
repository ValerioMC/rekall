use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, ErrorKind};
use std::path::{Path, PathBuf};

use rekall_core::token::codex_session_key;
use serde::Deserialize;

use super::SessionFiles;

#[derive(Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
enum FirstLine {
    SessionMeta { id: String, cwd: PathBuf },
    #[serde(other)]
    Other,
}

pub(super) fn rollouts_in(home: &Path, watched: &[PathBuf]) -> io::Result<Vec<SessionFiles>> {
    let watched: Vec<_> = watched.iter().map(|folder| canonical_or_same(folder)).collect();
    let mut sessions = Vec::new();
    for root in ["sessions", "archived_sessions"] {
        collect_rollouts(&home.join(root), &watched, &mut sessions)?;
    }
    sessions.sort_by(|left, right| right.bytes.cmp(&left.bytes));
    Ok(sessions)
}

fn collect_rollouts(folder: &Path, watched: &[PathBuf], sessions: &mut Vec<SessionFiles>) -> io::Result<()> {
    let entries = match fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(failure) if failure.kind() == ErrorKind::NotFound => return Ok(()),
        Err(failure) => return Err(failure),
    };
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let kind = entry.file_type()?;
        if kind.is_dir() {
            collect_rollouts(&path, watched, sessions)?;
        } else if kind.is_file() && is_rollout(&path) {
            match session_at(path, watched) {
                Ok(Some(session)) => sessions.push(session),
                Ok(None) => {}
                Err(failure) => tracing::debug!(%failure, "Could not read Codex rollout metadata"),
            }
        }
    }
    Ok(())
}

fn session_at(path: PathBuf, watched: &[PathBuf]) -> io::Result<Option<SessionFiles>> {
    let file = File::open(&path)?;
    let bytes = file.metadata()?.len();
    let mut first = String::new();
    BufReader::new(file).read_line(&mut first)?;
    let Ok(FirstLine::SessionMeta { id, cwd }) = serde_json::from_str(&first) else { return Ok(None) };
    let cwd = canonical_or_same(&cwd);
    if id.is_empty() || !watched.iter().any(|root| cwd.starts_with(root)) {
        return Ok(None);
    }
    Ok(Some(SessionFiles { session_id: codex_session_key(&id), main: path, subagents: Vec::new(), bytes }))
}

fn canonical_or_same(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn is_rollout(path: &Path) -> bool {
    path.file_name().and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("rollout-") && name.ends_with(".jsonl"))
}
