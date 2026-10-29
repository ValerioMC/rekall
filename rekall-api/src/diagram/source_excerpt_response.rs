use rekall_console::diagram::SourceExcerptView;
use serde::Serialize;

use crate::diagram::SourceLineResponse;

/// The lines of a span with a few around it. `highlightStart`/`highlightEnd` are the span
/// itself, null when the whole file was asked for.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceExcerptResponse {
    pub file: String,
    pub language: Option<String>,
    pub highlight_start: Option<u32>,
    pub highlight_end: Option<u32>,
    pub total_lines: u32,
    pub lines: Vec<SourceLineResponse>,
    pub truncated: bool,
}

impl From<SourceExcerptView> for SourceExcerptResponse {
    fn from(view: SourceExcerptView) -> Self {
        Self {
            file: view.file,
            language: view.language,
            highlight_start: view.highlight_start,
            highlight_end: view.highlight_end,
            total_lines: view.total_lines,
            lines: view.lines.into_iter().map(SourceLineResponse::from).collect(),
            truncated: view.truncated,
        }
    }
}
