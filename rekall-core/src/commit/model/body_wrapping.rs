use std::sync::LazyLock;

use regex::Regex;
use rekall_kernel::jstr;

/// Body lines are wrapped here, the width `git log` is read at.
pub const BODY_WIDTH: usize = 72;

/// `^(\s*)([-*+]|\d+[.)])\s+(.*)$` with `\s` and `\d` restricted to ASCII.
static LIST_ITEM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([ \t\n\x0B\x0C\r]*)([-*+]|[0-9]+[.)])[ \t\n\x0B\x0C\r]+(.*)$").expect("a valid pattern"));

static BLANK_RUNS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\n{3,}").expect("a valid pattern"));

pub(super) fn is_list_item(line: &str) -> bool {
    LIST_ITEM.is_match(line)
}

/// Reflows the body at the body width: the lines of a paragraph are joined and rewrapped, a
/// list item keeps its marker and a hanging indent, and blank lines between paragraphs stay.
pub(super) fn wrap(body: &str) -> String {
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
    for word in jstr::collapse_whitespace(jstr::strip(text)).split(' ') {
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
