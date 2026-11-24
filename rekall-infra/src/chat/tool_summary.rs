//! The one line a tool's result earns in the chat: what the tool did, from what `claude` reported
//! beside its text.

use super::wire::{WirePatchHunk, WireToolUseResult, WireWriteKind};

const FAILED_BASH_PREFIX: &str = "Error: Exit code ";

/// `None` when the result is of a shape Rekall does not summarise.
pub(super) fn summarise(result: &WireToolUseResult) -> Option<String> {
    match result {
        WireToolUseResult::Read { file } => Some(format!("Read {}", counted(file.num_lines, "line", "lines"))),
        WireToolUseResult::Write { r#type: WireWriteKind::Create, content, .. } => Some(lines_changed(content.lines().count() as u64, 0)),
        WireToolUseResult::Write { structured_patch, .. } | WireToolUseResult::Edit { structured_patch, .. } => {
            Some(lines_changed(prefixed_lines(structured_patch, '+'), prefixed_lines(structured_patch, '-')))
        }
        WireToolUseResult::Grep { mode, num_files, num_lines, num_matches } => Some(grep_summary(mode, *num_files, *num_lines, *num_matches)),
        WireToolUseResult::Glob { filenames } => Some(counted(filenames.len() as u64, "file", "files")),
        WireToolUseResult::Bash { interrupted: true, .. } => Some("interrupted".to_string()),
        WireToolUseResult::Bash { .. } => Some("exit code 0".to_string()),
        WireToolUseResult::Failure(text) => failed_exit_code(text).map(|code| format!("exit code {code}")),
        WireToolUseResult::Other(_) => None,
    }
}

fn grep_summary(mode: &str, files: u64, lines: Option<u64>, matches: Option<u64>) -> String {
    match (mode, lines, matches) {
        ("content", Some(lines), _) => counted(lines, "match", "matches"),
        ("count", _, Some(matches)) => counted(matches, "match", "matches"),
        _ => counted(files, "file", "files"),
    }
}

fn lines_changed(added: u64, removed: u64) -> String {
    format!("Added {}, removed {removed}", counted(added, "line", "lines"))
}

fn prefixed_lines(hunks: &[WirePatchHunk], prefix: char) -> u64 {
    hunks.iter().flat_map(|hunk| &hunk.lines).filter(|line| line.starts_with(prefix)).count() as u64
}

fn failed_exit_code(failure: &str) -> Option<u32> {
    let digits: String = failure.strip_prefix(FAILED_BASH_PREFIX)?.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

fn counted(count: u64, singular: &str, plural: &str) -> String {
    format!("{count} {}", if count == 1 { singular } else { plural })
}
