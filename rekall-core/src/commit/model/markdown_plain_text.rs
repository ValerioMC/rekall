use std::sync::LazyLock;

use regex::Regex;
use rekall_kernel::jstr;

use super::body_wrapping::is_list_item;

/// How much of a step's detail or a wrapup the derived body carries.
pub const DERIVED_BODY_MAX: usize = 800;

static SENTENCE_END: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[.!?]([ \t\n\x0B\x0C\r]|$)").expect("a valid pattern"));

static RULE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[-*_]{3,}$").expect("a valid pattern"));

static QUOTE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[ \t\n\x0B\x0C\r]*>[ \t\n\x0B\x0C\r]?").expect("a valid pattern"));

/// The body when the session wrote none: the opening paragraphs of the markdown that
/// describes the work, headings, emphasis and code fences dropped, cut at a paragraph or a
/// sentence.
pub(super) fn derived_body(markdown: Option<&str>) -> String {
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
    let list = lines.iter().any(|line| is_list_item(line));
    paragraphs.push(if list {
        lines.iter().map(|line| jstr::strip(line)).collect::<Vec<_>>().join("\n")
    } else {
        jstr::collapse_whitespace(jstr::strip(&lines.join(" ")))
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

#[cfg(test)]
#[path = "../../../tests/unit/commit/model/markdown_plain_text_tests.rs"]
mod tests;
