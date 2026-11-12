use std::sync::LazyLock;

use regex::Regex;
use rekall_kernel::jstr;

use crate::commit::{PendingChange, Subject};

use super::body_wrapping::wrap;
use super::change_classification::type_of;
use super::markdown_plain_text::derived_body;

/// Git shows the subject in full up to here; past it, tools wrap or cut.
pub const SUBJECT_MAX: usize = 72;

static CONVENTIONAL_PREFIX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(feat|fix|docs|test|refactor|chore|perf|build|ci|style|revert)(\([^)]*\))?!?: .+$").expect("a valid pattern")
});

pub struct CommitMessageGenerator;

impl CommitMessageGenerator {
    /// `session_message` is the message the claiming session wrote, or `None` to derive one.
    pub fn generate(subject: &Subject, changes: &[PendingChange], session_message: Option<&str>) -> String {
        let (session_subject, session_body) = parse_session(session_message);
        let subject_line = match &session_subject {
            None => Self::subject_line(subject, changes),
            Some(written) => session_subject_line(written, subject, changes),
        };
        let body = match session_body {
            None => derived_body(subject.description.as_deref()),
            Some(body) => body,
        };
        let mut message = format!("{subject_line}\n\n");
        if !jstr::is_blank(&body) {
            message.push_str(&wrap(&body));
            message.push_str("\n\n");
        }
        message.push_str("Refs: ");
        message.push_str(&subject.anchor);
        if let Some(number) = subject.step_number {
            message.push_str(&format!(", step {number}"));
        }
        message.push('\n');
        message
    }

    pub fn subject_line(subject: &Subject, changes: &[PendingChange]) -> String {
        fitted(&format!("{}: ", type_of(&subject.title, changes)), &jstr::collapse_whitespace(jstr::strip(&subject.title)))
    }
}

/// A session's own message, split into a subject and a body; either may be absent.
fn parse_session(message: Option<&str>) -> (Option<String>, Option<String>) {
    let Some(message) = message.filter(|m| !jstr::is_blank(m)) else {
        return (None, None);
    };
    let text = jstr::strip(message);
    match text.find('\n') {
        None => (Some(text.to_string()), None),
        Some(newline) => {
            let body = jstr::strip(&text[newline + 1..]);
            (Some(text[..newline].to_string()), if body.is_empty() { None } else { Some(body.to_string()) })
        }
    }
}

fn session_subject_line(written: &str, subject: &Subject, changes: &[PendingChange]) -> String {
    let line = jstr::collapse_whitespace(jstr::strip(written));
    if CONVENTIONAL_PREFIX.is_match(&line) {
        let colon = line.find(": ").expect("the prefix has one");
        return fitted(&line[..colon + 2], &line[colon + 2..]);
    }
    fitted(&format!("{}: ", type_of(&subject.title, changes)), &line)
}

fn fitted(prefix: &str, title: &str) -> String {
    let room = SUBJECT_MAX.saturating_sub(jstr::len(prefix));
    if jstr::len(title) > room {
        return format!("{prefix}{}…", jstr::strip_trailing(jstr::prefix(title, room.saturating_sub(1))));
    }
    format!("{prefix}{title}")
}

#[cfg(test)]
#[path = "../../../tests/unit/commit/model/commit_message_generator_tests.rs"]
mod tests;
