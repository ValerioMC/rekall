//! Writes the message for an automatic commit: a Conventional Commits subject and a prose body
//! that says what the change does, never which files it touched.
//!
//! The words come from the best source there is. A session that claims work can hand over its
//! own message, and that wins: its first line is the subject, the rest is the body. Without one,
//! the subject is the title of what was claimed and the body is drawn from the text that
//! describes it, the step's detail or the task's wrapup, turned from markdown into plain
//! paragraphs and cut to a few sentences. A final `Refs:` line names the task and step either way.
//!
//! The type is the session's when it wrote one. Otherwise the title decides `fix` and `refactor`,
//! and the files decide `docs` (every file is documentation) and `test` (every file is a test);
//! anything else is `feat`.

use std::sync::LazyLock;

use regex::Regex;
use rekall_common::jstr;

use super::PendingChange;

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

/// What the commit is for: the step or task claimed, as a title, where it lives, and the markdown
/// that describes it (a step's detail or the task's wrapup), which may be absent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Subject {
    pub title: String,
    pub anchor: String,
    pub step_number: Option<i32>,
    pub description: Option<String>,
}

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
mod tests {
    use super::super::PendingChangeKind;
    use super::*;

    const ANCHOR: &str = "project:vega task:report-builder";
    const DETAIL: &str = "## What this step does\n\nExpose **the weekly report** as a download, so the finance team stops asking for it by mail.\nThe endpoint streams the file instead of building it in memory.\n\n```java\nignored();\n```\n\nIt is read-only and needs no migration.\n";

    fn step() -> Subject {
        Subject { title: "Wire the export endpoint".into(), anchor: ANCHOR.into(), step_number: Some(3), description: Some(DETAIL.into()) }
    }

    fn mixed() -> Vec<PendingChange> {
        vec![
            PendingChange::new(PendingChangeKind::Added, "src/export/ExportController.java"),
            PendingChange::new(PendingChangeKind::Modified, "README.md"),
        ]
    }

    #[test]
    fn without_a_session_message_the_body_is_the_steps_detail_as_prose_and_no_file_is_listed() {
        let message = CommitMessageGenerator::generate(&step(), &mixed(), None);
        assert!(message.starts_with("feat: Wire the export endpoint\n\n"), "{message}");
        assert!(message.contains("Expose the weekly report as a download"));
        assert!(message.contains("It is read-only and needs no migration."));
        assert!(!message.contains("##") && !message.contains("**") && !message.contains("ignored();"));
        assert!(!message.contains("ExportController.java") && !message.contains("README.md") && !message.contains("files"));
        assert!(message.ends_with("\n\nRefs: project:vega task:report-builder, step 3\n"));
    }

    #[test]
    fn a_session_message_wins() {
        let written = "fix: stream the weekly report instead of buffering it\n\nThe export built the whole file in memory and ran out of heap on large months.\nIt now writes rows as they are read.\n";
        let message = CommitMessageGenerator::generate(&step(), &mixed(), Some(written));
        assert!(message.starts_with("fix: stream the weekly report instead of buffering it\n\n"));
        assert!(
            message.contains("ran out of heap on large\nmonths. It now writes rows as they are read."),
            "the session's lines are reflowed at 72 columns: {message}"
        );
        assert!(!message.contains("Expose the weekly"));
        assert!(message.ends_with("Refs: project:vega task:report-builder, step 3\n"));
    }

    #[test]
    fn a_bare_session_subject_gets_a_type_and_keeps_the_derived_body() {
        let message = CommitMessageGenerator::generate(&step(), &mixed(), Some("Stream the weekly report"));
        assert!(message.starts_with("feat: Stream the weekly report\n\n"));
        assert!(message.contains("Expose the weekly report"));
    }

    #[test]
    fn body_lines_are_wrapped_at_72_columns_and_list_items_keep_a_hanging_indent() {
        let written = format!("feat: x\n\n{}\n\n- {}", "word ".repeat(40), "item ".repeat(30));
        let message = CommitMessageGenerator::generate(&step(), &mixed(), Some(&written));
        assert!(message.split('\n').all(|line| jstr::len(line) <= 72));
        assert!(message.contains("\n- item") && message.contains("\n  item"));
    }

    #[test]
    fn a_long_detail_is_cut_at_a_sentence() {
        let detail = "This sentence is long enough to matter. ".repeat(40);
        let body = CommitMessageGenerator::derived_body(Some(&detail));
        assert!(jstr::len(&body) <= DERIVED_BODY_MAX);
        assert!(body.ends_with("matter."));
    }

    #[test]
    fn a_task_claim_with_no_wrapup_has_a_subject_and_the_refs_line() {
        let task = Subject { title: "Report builder".into(), anchor: ANCHOR.into(), step_number: None, description: None };
        let message = CommitMessageGenerator::generate(&task, &[PendingChange::new(PendingChangeKind::Deleted, "old.js")], None);
        assert_eq!(message, "feat: Report builder\n\nRefs: project:vega task:report-builder\n");
    }

    #[test]
    fn only_documentation_files_make_a_docs_commit_only_tests_a_test_commit() {
        let docs = [
            PendingChange::new(PendingChangeKind::Modified, "README.md"),
            PendingChange::new(PendingChangeKind::Added, "docs/export.txt"),
        ];
        assert_eq!(CommitMessageGenerator::type_of("Update guide", &docs), "docs");
        let tests = [
            PendingChange::new(PendingChangeKind::Added, "src/test/java/dev/vega/ExportTest.java"),
            PendingChange::new(PendingChangeKind::Modified, "ui/tests/unit/Export.spec.ts"),
        ];
        assert_eq!(CommitMessageGenerator::type_of("Cover export", &tests), "test");
        let mixed = [
            PendingChange::new(PendingChangeKind::Added, "src/test/java/dev/vega/ExportTest.java"),
            PendingChange::new(PendingChangeKind::Added, "src/main/java/dev/vega/Export.java"),
        ];
        assert_eq!(CommitMessageGenerator::type_of("Cover export", &mixed), "feat");
    }

    #[test]
    fn the_title_decides_fix_and_refactor_in_english_or_italian() {
        assert_eq!(CommitMessageGenerator::type_of("Fix the await review", &mixed()), "fix");
        assert_eq!(CommitMessageGenerator::type_of("Errore quando faccio accetto", &mixed()), "fix");
        assert_eq!(CommitMessageGenerator::type_of("Refactor the store", &mixed()), "refactor");
        assert_eq!(CommitMessageGenerator::type_of("Fixture loader", &mixed()), "feat");
    }

    #[test]
    fn the_subject_line_is_cut_at_72_characters_with_the_title_ellipsised() {
        let long = Subject { title: "A".repeat(100), anchor: ANCHOR.into(), step_number: Some(1), description: None };
        let subject = CommitMessageGenerator::subject_line(&long, &[]);
        assert_eq!(jstr::len(&subject), SUBJECT_MAX);
        assert!(subject.starts_with("feat: AAAA") && subject.ends_with('…'));
    }

    #[test]
    fn a_whitespace_heavy_title_is_collapsed_to_single_spaces() {
        let messy = Subject { title: "  Wire   the\nexport  ".into(), anchor: ANCHOR.into(), step_number: Some(1), description: None };
        assert_eq!(CommitMessageGenerator::subject_line(&messy, &[]), "feat: Wire the export");
    }
}
