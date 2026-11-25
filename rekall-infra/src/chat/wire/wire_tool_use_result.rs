use serde::de::IgnoredAny;
use serde::Deserialize;

use super::{WirePatchHunk, WireReadFile, WireWriteKind};

/// The `tool_use_result` of a `user` line: what a tool reported beside its text. Its shape is the
/// tool's own and carries no tag, so each variant is told apart by the fields it requires, tried in
/// this order (`Write` before `Edit`, `Grep` before `Glob`). A failed
/// call reports a plain string; any shape Rekall does not summarise is `Other`.
#[derive(Debug, Deserialize)]
#[serde(untagged, rename_all_fields = "camelCase")]
pub(in crate::chat) enum WireToolUseResult {
    Read { file: WireReadFile },
    Write { r#type: WireWriteKind, content: String, structured_patch: Vec<WirePatchHunk> },
    Edit { structured_patch: Vec<WirePatchHunk> },
    Grep { mode: String, num_files: u64, num_lines: Option<u64>, num_matches: Option<u64> },
    Glob { filenames: Vec<String> },
    Bash { interrupted: bool },
    Failure(String),
    Other(IgnoredAny),
}
