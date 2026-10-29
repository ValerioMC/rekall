use crate::diagram::SourceLineView;

/// The lines of a span with a few around it. `highlightStart`/`highlightEnd` are the span
/// itself, null when the whole file was asked for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceExcerptView {
    pub file: String,
    pub language: Option<String>,
    pub highlight_start: Option<u32>,
    pub highlight_end: Option<u32>,
    pub total_lines: u32,
    pub lines: Vec<SourceLineView>,
    pub truncated: bool,
}
