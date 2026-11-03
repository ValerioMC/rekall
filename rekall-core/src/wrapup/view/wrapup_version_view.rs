use rekall_kernel::{jstr, Id, Instant};
use rekall_store::wrapup::wrapup_version;
use rekall_store::WrapupAuthor;
use serde::Serialize;

const PREVIEW_CHARACTERS: usize = 160;

/// A past text of a wrapup, without the text: what the Versions list shows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WrapupVersionView {
    pub id: Id,
    pub task_id: Id,
    pub number: i32,
    pub written_by: WrapupAuthor,
    pub written_at: Instant,
    pub length: usize,
    pub preview: String,
    pub commit_hash: Option<String>,
}

impl WrapupVersionView {
    pub fn of(version: &wrapup_version::Model) -> Self {
        Self {
            id: version.id,
            task_id: version.task_id,
            number: version.number,
            written_by: version.written_by,
            written_at: version.written_at,
            length: jstr::len(&version.body_markdown),
            preview: preview_of(&version.body_markdown),
            commit_hash: version.commit_hash.clone(),
        }
    }
}

/// One version with its whole text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WrapupVersionDetail {
    #[serde(flatten)]
    pub version: WrapupVersionView,
    pub body_markdown: String,
}

impl WrapupVersionDetail {
    pub fn of(version: &wrapup_version::Model) -> Self {
        Self { version: WrapupVersionView::of(version), body_markdown: version.body_markdown.clone() }
    }
}

fn preview_of(body: &str) -> String {
    let flat = jstr::collapse_whitespace(body);
    let flat = jstr::strip(&flat);
    if jstr::len(flat) <= PREVIEW_CHARACTERS {
        return flat.to_string();
    }
    format!("{}…", jstr::strip_trailing(jstr::prefix(flat, PREVIEW_CHARACTERS)))
}
