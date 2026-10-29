//! The text rules every cap, excerpt and blank check in Rekall is measured with.
//!
//! Lengths and indices count UTF-16 code units, the unit the console's JavaScript counts in, so a
//! cap (a wrapup is 20,000 characters, a note 100,000) refuses exactly what the console's counter
//! shows as too long. Rust's `len()` counts bytes and would disagree on any accented letter or
//! emoji. Blank means only whitespace by `is_whitespace`, which leaves non-breaking spaces out.

/// Space, line and paragraph separators except the non-breaking ones, plus the ASCII controls
/// `\t \n \u000B \f \r` and `\u001C`-`\u001F`.
pub fn is_whitespace(c: char) -> bool {
    match c {
        '\t' | '\n' | '\u{000B}' | '\u{000C}' | '\r' | '\u{001C}'..='\u{001F}' => true,
        '\u{00A0}' | '\u{2007}' | '\u{202F}' => false,
        ' ' | '\u{1680}' | '\u{2000}'..='\u{2006}' | '\u{2008}'..='\u{200A}' | '\u{205F}' | '\u{3000}' => true,
        '\u{2028}' | '\u{2029}' => true,
        _ => false,
    }
}

/// Without leading and trailing whitespace.
pub fn strip(text: &str) -> &str {
    text.trim_matches(is_whitespace)
}

/// Without trailing whitespace.
pub fn strip_trailing(text: &str) -> &str {
    text.trim_end_matches(is_whitespace)
}

/// Without leading whitespace.
pub fn strip_leading(text: &str) -> &str {
    text.trim_start_matches(is_whitespace)
}

/// Empty, or whitespace only.
pub fn is_blank(text: &str) -> bool {
    text.chars().all(is_whitespace)
}

/// Absent, empty, or whitespace only.
pub fn is_null_or_blank(text: Option<&str>) -> bool {
    text.is_none_or(is_blank)
}

/// Without leading and trailing code points up to and including U+0020, and nothing else.
pub fn trim(text: &str) -> &str {
    text.trim_matches(|c: char| c <= ' ')
}

/// The length in UTF-16 code units.
pub fn len(text: &str) -> usize {
    text.chars().map(char::len_utf16).sum()
}

/// The byte offset of UTF-16 index `index`, clamped to the text. An index inside a surrogate pair
/// is moved back to the start of that character, so a cut never splits one.
fn byte_offset(text: &str, index: usize) -> usize {
    let mut units = 0;
    for (offset, c) in text.char_indices() {
        let width = c.len_utf16();
        if units + width > index {
            return offset;
        }
        units += width;
    }
    text.len()
}

/// The part between UTF-16 indices `start` and `end`, clamped to the text.
pub fn substring(text: &str, start: usize, end: usize) -> &str {
    let from = byte_offset(text, start);
    let to = byte_offset(text, end).max(from);
    &text[from..to]
}

/// The first `end` UTF-16 units.
pub fn prefix(text: &str, end: usize) -> &str {
    substring(text, 0, end)
}

/// Everything from UTF-16 index `start`.
pub fn suffix(text: &str, start: usize) -> &str {
    &text[byte_offset(text, start)..]
}

/// Where `needle` first starts, as a UTF-16 index.
pub fn index_of(text: &str, needle: &str) -> Option<usize> {
    text.find(needle).map(|byte| len(&text[..byte]))
}

/// Where `needle` first starts at or after UTF-16 index `from`.
pub fn index_of_from(text: &str, needle: &str, from: usize) -> Option<usize> {
    let start = byte_offset(text, from);
    text[start..].find(needle).map(|byte| len(&text[..start + byte]))
}

/// Where the last `needle` starts at or before UTF-16 index `from`.
pub fn last_index_of_from(text: &str, needle: &str, from: usize) -> Option<usize> {
    let limit = byte_offset(text, from);
    let mut best = None;
    for (byte, _) in text.match_indices(needle) {
        if byte > limit {
            break;
        }
        best = Some(len(&text[..byte]));
    }
    best
}

/// Every run of ASCII whitespace (`[ \t\n\x0B\f\r]`) as one space.
pub fn collapse_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_run = false;
    for c in text.chars() {
        if is_ascii_space(c) {
            if !in_run {
                out.push(' ');
                in_run = true;
            }
        } else {
            out.push(c);
            in_run = false;
        }
    }
    out
}

/// ASCII whitespace: `[ \t\n\x0B\f\r]`.
pub fn is_ascii_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\u{000B}' | '\u{000C}' | '\r')
}

/// Split on `\n`, `\r` or `\r\n`, with no empty trailing line.
pub fn lines(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                out.push(&text[start..i]);
                i += 1;
                start = i;
            }
            b'\r' => {
                out.push(&text[start..i]);
                i += if bytes.get(i + 1) == Some(&b'\n') { 2 } else { 1 };
                start = i;
            }
            _ => i += 1,
        }
    }
    if start < bytes.len() {
        out.push(&text[start..]);
    }
    out
}

/// Split on `\r\n` or any single line break. Trailing empty parts are dropped unless
/// `keep_trailing` is set.
pub fn split_linebreaks(text: &str, keep_trailing: bool) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        let breaks = matches!(c, '\n' | '\u{000B}' | '\u{000C}' | '\r' | '\u{0085}' | '\u{2028}' | '\u{2029}');
        if !breaks {
            continue;
        }
        out.push(&text[start..at]);
        let mut next = at + c.len_utf8();
        if c == '\r' {
            if let Some(&(_, '\n')) = chars.peek() {
                chars.next();
                next += 1;
            }
        }
        start = next;
    }
    out.push(&text[start..]);
    if !keep_trailing {
        while out.len() > 1 && out.last() == Some(&"") {
            out.pop();
        }
        if out.len() == 1 && out[0].is_empty() && !text.is_empty() {
            out.clear();
        }
    }
    out
}

/// Equal after upper-casing and then lower-casing each character.
pub fn equals_ignore_case(a: &str, b: &str) -> bool {
    let mut left = a.chars();
    let mut right = b.chars();
    loop {
        match (left.next(), right.next()) {
            (None, None) => return true,
            (Some(x), Some(y)) => {
                if x == y {
                    continue;
                }
                let ux = single_upper(x);
                let uy = single_upper(y);
                if ux == uy {
                    continue;
                }
                if single_lower(ux) == single_lower(uy) {
                    continue;
                }
                return false;
            }
            _ => return false,
        }
    }
}

/// Character by character, each upper-cased and then lower-cased before they are compared, then
/// the shorter first.
pub fn compare_ignore_case(a: &str, b: &str) -> std::cmp::Ordering {
    let mut left = a.chars();
    let mut right = b.chars();
    loop {
        match (left.next(), right.next()) {
            (None, None) => return std::cmp::Ordering::Equal,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some(x), Some(y)) => {
                if x == y {
                    continue;
                }
                let lx = single_lower(single_upper(x));
                let ly = single_lower(single_upper(y));
                if lx != ly {
                    return lx.cmp(&ly);
                }
            }
        }
    }
}

fn single_upper(c: char) -> char {
    let mut upper = c.to_uppercase();
    match (upper.next(), upper.next()) {
        (Some(u), None) => u,
        _ => c,
    }
}

fn single_lower(c: char) -> char {
    let mut lower = c.to_lowercase();
    match (lower.next(), lower.next()) {
        (Some(l), None) => l,
        _ => c,
    }
}

/// Lower case with no locale rules.
pub fn lower(text: &str) -> String {
    text.to_lowercase()
}

#[cfg(test)]
#[path = "../tests/unit/jstr_tests.rs"]
mod tests;
