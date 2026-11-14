use serde::Serialize;

use super::DiffLineView;

/// One `@@` block of a file's patch: where it sits in the old and new file, the nearest enclosing
/// line git printed after it, and its lines.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffHunkView {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub context: String,
    pub lines: Vec<DiffLineView>,
}
