use std::sync::LazyLock;

use regex::Regex;
use rekall_common::jstr;

use super::PendingChange;

use super::Subject;

/// Git shows the subject in full up to here; past it, tools wrap or cut.
pub const SUBJECT_MAX: usize = 72;
/// Body lines are wrapped here, the width `git log` is read at.
pub const BODY_WIDTH: usize = 72;
/// How much of a step's detail or a wrapup the derived body carries.
pub const DERIVED_BODY_MAX: usize = 800;

static DOCUMENTATION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(^|/)(docs?|documentation)/|\.(md|mdx|markdown|rst|txt|adoc)$").unwrap());

static TEST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(^|/)(tests?|__tests__|spec|e2e)/|\.(spec|test)\.[a-z]+$|Tests?\.java$|_test\.(go|py)$|test_[^/]*\.py$")
        .unwrap()
});

static CONVENTIONAL_PREFIX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(feat|fix|docs|test|refactor|chore|perf|build|ci|style|revert)(\([^)]*\))?!?: .+$").unwrap()
});

static FIX_TITLE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(fix|bug|hotfix|correggi|corregge|correzione|risolvi|risolve|errore|problema)\b").unwrap()
});

static REFACTOR_TITLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^(refactor|refactoring|riorganizza|ripulisci)\b").unwrap());

/// `^(\s*)([-*+]|\d+[.)])\s+(.*)$` with Java's ASCII `\s` and `\d`.
static LIST_ITEM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([ \t\n\x0B\x0C\r]*)([-*+]|[0-9]+[.)])[ \t\n\x0B\x0C\r]+(.*)$").unwrap());

static SENTENCE_END: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[.!?]([ \t\n\x0B\x0C\r]|$)").unwrap());

static RULE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[-*_]{3,}$").unwrap());

static QUOTE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[ \t\n\x0B\x0C\r]*>[ \t\n\x0B\x0C\r]?").unwrap());

static BLANK_RUNS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\n{3,}").unwrap());

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
            None => Self::derived_body(subject.description.as_deref()),
            Some(body) => body,
        };
        let mut message = format!("{subject_line}\n\n");
        if !jstr::is_blank(&body) {
            message.push_str(&Self::wrap(&body));
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
        fitted(&format!("{}: ", Self::type_of(&subject.title, changes)), &collapsed(Some(&subject.title)))
    }

    pub fn type_of(title: &str, changes: &[PendingChange]) -> &'static str {
        let plain = collapsed(Some(title));
        if FIX_TITLE.is_match(&plain) {
            return "fix";
        }
        if REFACTOR_TITLE.is_match(&plain) {
            return "refactor";
        }
        if !changes.is_empty() && changes.iter().all(|change| DOCUMENTATION.is_match(&change.path)) {
            return "docs";
        }
        if !changes.is_empty() && changes.iter().all(|change| TEST.is_match(&change.path)) {
            return "test";
        }
        "feat"
    }

    /// The body when the session wrote none: the opening paragraphs of the markdown that
    /// describes the work, headings, emphasis and code fences dropped, cut at a paragraph or a
    /// sentence.
    pub fn derived_body(markdown: Option<&str>) -> String {
        let Some(markdown) = markdown.filter(|m| !jstr::is_blank(m)) else {
            return String::new();
        };
        let mut body = String::new();
        for paragraph in plain_paragraphs(markdown) {
            let separator = if body.is_empty() { 0 } else { 2 };
            if jstr::len(&body) + separator + jstr::len(&paragraph) <= DERIVED_BODY_MAX {
                if separator > 0 {
                    body.push_str("\n\n");
                }
                body.push_str(&paragraph);
                continue;
            }
            if body.is_empty() {
                body.push_str(&cut_at_sentence(&paragraph, DERIVED_BODY_MAX));
            }
            break;
        }
        body
    }

    /// Reflows the body at the body width: the lines of a paragraph are joined and rewrapped, a
    /// list item keeps its marker and a hanging indent, and blank lines between paragraphs stay.
    pub fn wrap(body: &str) -> String {
        let mut out: Vec<String> = Vec::new();
        let mut marker: Option<String> = None;
        let mut block = String::new();
        for line in jstr::split_linebreaks(jstr::strip(body), true) {
            let item = LIST_ITEM.captures(line);
            if jstr::is_blank(line) || item.is_some() {
                flush_block(&mut block, marker.as_deref(), &mut out);
                marker = None;
                if jstr::is_blank(line) {
                    out.push(String::new());
                    continue;
                }
                let item = item.expect("matched above");
                marker = Some(format!("{}{} ", &item[1], &item[2]));
                block.push_str(&item[3]);
                continue;
            }
            if !block.is_empty() {
                block.push(' ');
            }
            block.push_str(jstr::strip(line));
        }
        flush_block(&mut block, marker.as_deref(), &mut out);
        BLANK_RUNS.replace_all(&out.join("\n"), "\n\n").into_owned()
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
    let line = collapsed(Some(written));
    if CONVENTIONAL_PREFIX.is_match(&line) {
        let colon = line.find(": ").expect("the prefix has one");
        return fitted(&line[..colon + 2], &line[colon + 2..]);
    }
    fitted(&format!("{}: ", CommitMessageGenerator::type_of(&subject.title, changes)), &line)
}

fn fitted(prefix: &str, title: &str) -> String {
    let room = SUBJECT_MAX.saturating_sub(jstr::len(prefix));
    if jstr::len(title) > room {
        return format!("{prefix}{}…", jstr::strip_trailing(jstr::prefix(title, room.saturating_sub(1))));
    }
    format!("{prefix}{title}")
}

fn plain_paragraphs(markdown: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut current: Vec<String> = Vec::new();
    let mut in_fence = false;
    for raw in jstr::split_linebreaks(jstr::strip(markdown), false) {
        let line = jstr::strip_trailing(raw);
        let stripped = jstr::strip(line);
        if stripped.starts_with("```") || stripped.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if jstr::is_blank(line) || stripped.starts_with('#') || RULE.is_match(stripped) {
            flush(&mut current, &mut paragraphs);
            continue;
        }
        current.push(unemphasised(line));
    }
    flush(&mut current, &mut paragraphs);
    paragraphs
}

fn flush(lines: &mut Vec<String>, paragraphs: &mut Vec<String>) {
    if lines.is_empty() {
        return;
    }
    let list = lines.iter().any(|line| LIST_ITEM.is_match(line));
    paragraphs.push(if list {
        lines.iter().map(|line| jstr::strip(line)).collect::<Vec<_>>().join("\n")
    } else {
        collapsed(Some(&lines.join(" ")))
    });
    lines.clear();
}

fn unemphasised(line: &str) -> String {
    let plain = line.replace("**", "").replace("__", "");
    QUOTE.replace(&plain, "").into_owned()
}

fn cut_at_sentence(text: &str, max: usize) -> String {
    let head = jstr::prefix(text, max.min(jstr::len(text)));
    let last_end = SENTENCE_END.find_iter(head).last().map(|m| jstr::len(&head[..m.start()]) + 1);
    if let Some(last_end) = last_end {
        if last_end > max / 3 {
            return jstr::prefix(head, last_end).to_string();
        }
    }
    let cut = match jstr::last_index_of_from(head, " ", jstr::len(head)) {
        Some(space) if space > 0 => jstr::prefix(head, space),
        _ => head,
    };
    format!("{}…", jstr::strip_trailing(cut))
}

fn flush_block(block: &mut String, marker: Option<&str>, out: &mut Vec<String>) {
    if block.is_empty() {
        return;
    }
    match marker {
        None => out.extend(wrap_line(block, "", "")),
        Some(marker) => out.extend(wrap_line(block, marker, &" ".repeat(jstr::len(marker)))),
    }
    block.clear();
}

fn wrap_line(text: &str, first_indent: &str, next_indent: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = first_indent.to_string();
    let mut content_start = jstr::len(first_indent);
    for word in collapsed(Some(text)).split(' ') {
        if word.is_empty() {
            continue;
        }
        let mut empty = jstr::len(&line) == content_start;
        if !empty && jstr::len(&line) + 1 + jstr::len(word) > BODY_WIDTH {
            lines.push(line);
            line = next_indent.to_string();
            content_start = jstr::len(next_indent);
            empty = true;
        }
        if !empty {
            line.push(' ');
        }
        line.push_str(word);
    }
    lines.push(line);
    lines
}

fn collapsed(text: Option<&str>) -> String {
    match text {
        None => String::new(),
        Some(text) => jstr::collapse_whitespace(jstr::strip(text)),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/commit/commit_message_generator_tests.rs"]
mod tests;
