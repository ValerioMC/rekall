//! The `java.lang.String` semantics the domain rules were written against.
//!
//! The caps (a wrapup is 20,000 characters, a note 100,000), the truncation points and the
//! excerpt windows were all measured with `String.length()`, which counts UTF-16 code units, and
//! the blanks were judged with `strip()`/`isBlank()`, which use `Character.isWhitespace`. Rust's
//! `len()` counts bytes and `trim()` uses the Unicode `White_Space` property, and both differ on
//! real text (an accented letter, a non-breaking space, an emoji). These helpers keep the Java
//! arithmetic, so a limit refuses the same text it refused before.

/// `Character.isWhitespace`: space, line and paragraph separators except the non-breaking ones,
/// plus the ASCII controls `\t \n \u000B \f \r` and `\u001C`-`\u001F`.
pub fn is_whitespace(c: char) -> bool {
    match c {
        '\t' | '\n' | '\u{000B}' | '\u{000C}' | '\r' | '\u{001C}'..='\u{001F}' => true,
        '\u{00A0}' | '\u{2007}' | '\u{202F}' => false,
        ' ' | '\u{1680}' | '\u{2000}'..='\u{2006}' | '\u{2008}'..='\u{200A}' | '\u{205F}' | '\u{3000}' => true,
        '\u{2028}' | '\u{2029}' => true,
        _ => false,
    }
}

/// `String.strip()`.
pub fn strip(text: &str) -> &str {
    text.trim_matches(is_whitespace)
}

/// `String.stripTrailing()`.
pub fn strip_trailing(text: &str) -> &str {
    text.trim_end_matches(is_whitespace)
}

/// `String.stripLeading()`.
pub fn strip_leading(text: &str) -> &str {
    text.trim_start_matches(is_whitespace)
}

/// `String.isBlank()`.
pub fn is_blank(text: &str) -> bool {
    text.chars().all(is_whitespace)
}

/// `Option<String>` read the way `value == null || value.isBlank()` reads.
pub fn is_null_or_blank(text: Option<&str>) -> bool {
    text.is_none_or(is_blank)
}

/// `String.trim()`: every code point up to and including U+0020, and nothing else.
pub fn trim(text: &str) -> &str {
    text.trim_matches(|c: char| c <= ' ')
}

/// `String.length()`: UTF-16 code units.
pub fn len(text: &str) -> usize {
    text.chars().map(char::len_utf16).sum()
}

/// The byte offset of UTF-16 index `index`, clamped to the text. An index inside a surrogate pair
/// (where Java would split the pair) is moved back to the start of that character.
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

/// `String.substring(start, end)` in UTF-16 indices, clamped to the text.
pub fn substring(text: &str, start: usize, end: usize) -> &str {
    let from = byte_offset(text, start);
    let to = byte_offset(text, end).max(from);
    &text[from..to]
}

/// `String.substring(0, end)`.
pub fn prefix(text: &str, end: usize) -> &str {
    substring(text, 0, end)
}

/// `String.substring(start)`.
pub fn suffix(text: &str, start: usize) -> &str {
    &text[byte_offset(text, start)..]
}

/// `String.indexOf(needle)` as a UTF-16 index.
pub fn index_of(text: &str, needle: &str) -> Option<usize> {
    text.find(needle).map(|byte| len(&text[..byte]))
}

/// `String.indexOf(needle, from)` as a UTF-16 index.
pub fn index_of_from(text: &str, needle: &str, from: usize) -> Option<usize> {
    let start = byte_offset(text, from);
    text[start..].find(needle).map(|byte| len(&text[..start + byte]))
}

/// `String.lastIndexOf(needle, from)`: the last match starting at or before UTF-16 index `from`.
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

/// `text.replaceAll("\\s+", " ")`, where `\s` is Java's ASCII class `[ \t\n\x0B\f\r]`.
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

/// Java regex `\s` without `UNICODE_CHARACTER_CLASS`.
pub fn is_ascii_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\u{000B}' | '\u{000C}' | '\r')
}

/// `String.lines()`: split on `\n`, `\r` or `\r\n`, with no empty trailing line.
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

/// `text.split("\\R")` (with Java's dropping of trailing empty strings when `keep_trailing` is
/// false, and `split("\\R", -1)` when it is true). `\R` is `\r\n` or any single line break.
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

/// `String.equalsIgnoreCase`: equal after upper-casing and then lower-casing each character.
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

/// `String.CASE_INSENSITIVE_ORDER`: character by character, each upper-cased and then
/// lower-cased before they are compared, then the shorter first.
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

/// `String.toLowerCase()` as the Java code used it (root locale in practice).
pub fn lower(text: &str) -> String {
    text.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_keeps_a_non_breaking_space_like_java() {
        assert_eq!(strip("\u{00A0}x "), "\u{00A0}x");
        assert!(!is_blank("\u{00A0}"));
        assert!(is_blank(" \t\n"));
    }

    #[test]
    fn length_counts_utf16_units() {
        assert_eq!(len("abc"), 3);
        assert_eq!(len("é"), 1);
        assert_eq!(len("😀"), 2);
        assert_eq!(prefix("a😀b", 2), "a");
        assert_eq!(prefix("a😀b", 3), "a😀");
    }

    #[test]
    fn lines_split_like_java() {
        assert_eq!(lines("a\nb\r\nc\rd\n"), vec!["a", "b", "c", "d"]);
        assert_eq!(lines(""), Vec::<&str>::new());
        assert_eq!(lines("\n\nx"), vec!["", "", "x"]);
    }

    #[test]
    fn split_on_line_breaks_drops_trailing_empties_unless_asked() {
        assert_eq!(split_linebreaks("a\n\nb\n\n", false), vec!["a", "", "b"]);
        assert_eq!(split_linebreaks("a\n\nb\n", true), vec!["a", "", "b", ""]);
        assert_eq!(split_linebreaks("a\r\nb", false), vec!["a", "b"]);
    }

    #[test]
    fn collapses_ascii_whitespace_only() {
        assert_eq!(collapse_whitespace("a \t\n b\u{00A0}c"), "a b\u{00A0}c");
    }

    #[test]
    fn finds_like_java() {
        assert_eq!(index_of("héllo wörld", "wörld"), Some(6));
        assert_eq!(last_index_of_from("a b c d", " ", 4), Some(3));
        assert_eq!(index_of_from("a b c d", " ", 2), Some(3));
        assert!(!equals_ignore_case("Straße", "STRASSE"));
        assert!(equals_ignore_case("Hello", "hELLO"));
    }

    #[test]
    fn orders_ignoring_case_like_java() {
        use std::cmp::Ordering::*;
        assert_eq!(compare_ignore_case("app.ts", "README.md"), Less);
        assert_eq!(compare_ignore_case("Docs", "docs"), Equal);
        assert_eq!(compare_ignore_case("src", "Src2"), Less);
        assert_eq!(compare_ignore_case(".env", "app"), Less);
    }
}
