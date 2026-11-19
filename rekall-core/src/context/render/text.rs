//! The helpers every section writes through, and the limits a rendered section keeps to.

use rekall_kernel::jstr;

pub const MAX_DOCUMENT_CHARACTERS: usize = 20_000;

/// A diff is the one thing here worth more room than a note: it is what the session was handed to read.
pub const MAX_DIFF_CHARACTERS: usize = 60_000;

/// How much of a reference note's opening line travels with it.
pub const REFERENCE_LINE_MAX: usize = 200;

pub(super) fn attribute(value: &str) -> String {
    value.replace('"', "'")
}

pub(super) fn indent(body: &str) -> String {
    jstr::lines(body)
        .into_iter()
        .map(|line| if jstr::is_blank(line) { String::new() } else { format!("  {line}") })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn opening_line(body: &str) -> String {
    let first = jstr::lines(body)
        .into_iter()
        .map(jstr::strip)
        .find(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with("```"))
        .unwrap_or("");
    if jstr::len(first) <= REFERENCE_LINE_MAX {
        first.to_string()
    } else {
        format!("{}…", jstr::strip_trailing(jstr::prefix(first, REFERENCE_LINE_MAX)))
    }
}

pub(super) fn truncate(body: Option<&str>, limit: usize) -> String {
    let Some(body) = body else {
        return String::new();
    };
    let length = jstr::len(body);
    if length <= limit {
        return body.to_string();
    }
    format!("{}\n\n[truncated: {limit} of {length} characters shown]", jstr::prefix(body, limit))
}
