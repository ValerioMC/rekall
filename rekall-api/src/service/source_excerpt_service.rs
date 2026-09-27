//! Reads the code a diagram element points at, so the console can show CONCEPT → CODE without
//! leaving the page. It only ever reads inside the diagram's project folder: a path that climbs
//! out of it, through `..` or a symlink, is refused.

use std::fs;
use std::path::{Path, PathBuf};

use rekall_common::{jstr, Id, RekallError, Result};
use rekall_repository::repository as repo;
use rekall_service::{in_read, Services};

use crate::dto::{SourceExcerptResponse, SourceLineResponse};

/// Lines shown either side of the span, so it reads in its surroundings.
const CONTEXT_LINES: u32 = 4;
/// The most lines one excerpt carries; a longer span is cut and flagged.
pub const MAX_LINES: u32 = 400;
/// A file larger than this is not source a person reads, and is refused.
const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Clone)]
pub struct SourceExcerptService {
    services: Services,
}

impl SourceExcerptService {
    pub fn new(services: Services) -> Self {
        Self { services }
    }

    pub async fn excerpt(&self, diagram_id: Id, file: &str, start: Option<u32>, end: Option<u32>) -> Result<SourceExcerptResponse> {
        let folder = self.project_folder(diagram_id).await?;
        let file = file.to_string();
        tokio::task::spawn_blocking(move || read_excerpt(&folder, &file, start, end))
            .await
            .map_err(|failed| RekallError::internal("IllegalStateException", failed))?
    }

    async fn project_folder(&self, diagram_id: Id) -> Result<PathBuf> {
        in_read!(&self.services.ctx, |tx| {
            let row = repo::diagram::find_by_id(tx.db(), diagram_id).await?.ok_or_else(|| RekallError::not_found("Diagram", diagram_id))?;
            let project = repo::project::find_by_id(tx.db(), row.project_id).await?.ok_or_else(|| RekallError::not_found("Project", row.project_id))?;
            let folder = project.repo_folder.filter(|folder| !jstr::is_blank(folder)).ok_or_else(|| {
                RekallError::illegal("This diagram's project has no folder, so its code cannot be shown. Set one on the project page.")
            })?;
            Ok::<_, RekallError>(PathBuf::from(jstr::strip(&folder)))
        })
    }
}

/// The excerpt of `file` under `folder`, confined to it, with [`CONTEXT_LINES`] around the span.
fn read_excerpt(folder: &Path, file: &str, start: Option<u32>, end: Option<u32>) -> Result<SourceExcerptResponse> {
    let path = confined(folder, file)?;
    let size = fs::metadata(&path).map_err(|_| missing(file))?.len();
    if size > MAX_FILE_BYTES {
        return Err(RekallError::illegal(format!("{file} is {size} bytes, too large to show.")));
    }
    let bytes = fs::read(&path).map_err(|_| missing(file))?;
    let text = String::from_utf8_lossy(&bytes);
    let all: Vec<&str> = text.lines().collect();
    let total = u32::try_from(all.len()).unwrap_or(u32::MAX);

    let highlight_start = start.map(|line| line.clamp(1, total.max(1)));
    let highlight_end = highlight_start.map(|first| end.unwrap_or(first).clamp(first, total.max(first)));
    let (first, last) = match (highlight_start, highlight_end) {
        (Some(first), Some(last)) => (first.saturating_sub(CONTEXT_LINES).max(1), last.saturating_add(CONTEXT_LINES).min(total)),
        _ => (1, total),
    };
    let last_shown = last.min(first.saturating_add(MAX_LINES - 1));
    let lines = (first..=last_shown)
        .filter_map(|number| all.get((number - 1) as usize).map(|text| SourceLineResponse { number, text: text.to_string() }))
        .collect();
    Ok(SourceExcerptResponse {
        file: file.to_string(),
        language: language_of(file),
        highlight_start,
        highlight_end,
        total_lines: total,
        lines,
        truncated: last_shown < last,
    })
}

/// `file` resolved under `folder`, refused unless its real path still lies inside it.
fn confined(folder: &Path, file: &str) -> Result<PathBuf> {
    let root = folder.canonicalize().map_err(|_| RekallError::not_found_msg(format!("The project folder {} is not there.", folder.display())))?;
    let requested = Path::new(file.trim());
    let joined = if requested.is_absolute() { requested.to_path_buf() } else { root.join(requested) };
    let real = joined.canonicalize().map_err(|_| missing(file))?;
    if !real.starts_with(&root) {
        return Err(RekallError::illegal(format!("{file} is outside the project folder.")));
    }
    if !real.is_file() {
        return Err(RekallError::illegal(format!("{file} is not a file.")));
    }
    Ok(real)
}

fn missing(file: &str) -> RekallError {
    RekallError::not_found_msg(format!("There is no file {file} in the project folder."))
}

/// A highlight.js language name from the extension, when it is one the console registers.
fn language_of(file: &str) -> Option<String> {
    let extension = Path::new(file).extension()?.to_str()?.to_ascii_lowercase();
    let language = match extension.as_str() {
        "rs" => "rust",
        "ts" | "tsx" | "mts" => "typescript",
        "js" | "jsx" | "mjs" | "cjs" => "javascript",
        "vue" | "html" | "xml" => "xml",
        "py" => "python",
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "go" => "go",
        "sql" => "sql",
        "json" => "json",
        "yml" | "yaml" => "yaml",
        "toml" => "ini",
        "sh" | "zsh" | "bash" => "bash",
        "css" => "css",
        "md" => "markdown",
        _ => return None,
    };
    Some(language.to_string())
}

#[cfg(test)]
#[path = "../../tests/unit/service/source_excerpt_service_tests.rs"]
mod tests;
