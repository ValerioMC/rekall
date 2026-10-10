use crate::document::DiagramKind;

/// Sequence lines that open a statement other than a message, even when their text holds an arrow.
const SEQUENCE_STATEMENTS: [&str; 22] = [
    "note", "participant", "actor", "create", "destroy", "box", "loop", "alt", "else", "opt", "par", "and", "critical",
    "option", "break", "rect", "end", "activate", "deactivate", "autonumber", "link", "links",
];

/// Every sequence arrow holds one of these: `->` also covers `->>`, `-->>` and `<<->>`.
const SEQUENCE_ARROWS: [&str; 3] = ["->", "-x", "-)"];

/// A sheet's Mermaid source, read only as far as the links need: the declaration line, after
/// any front matter, comments and directives, and the statements that follow it.
#[derive(Clone, Copy, Debug)]
pub struct MermaidText<'a> {
    text: &'a str,
}

impl<'a> MermaidText<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }

    pub fn declared_kind(&self) -> Option<DiagramKind> {
        let (declaration, _) = self.split()?;
        DiagramKind::of_keyword(declaration.split_whitespace().next()?)
    }

    /// Lines that send a message, in order: what `autonumber` would number and what `#n` counts.
    pub fn message_count(&self) -> u32 {
        let count = self.statements().filter(|line| is_message(line)).count();
        u32::try_from(count).unwrap_or(u32::MAX)
    }

    /// Whether `id` appears as a whole token after the declaration, which is where every element
    /// a sheet draws is named.
    pub fn mentions(&self, id: &str) -> bool {
        !id.is_empty() && self.statements().any(|line| contains_token(line, id))
    }

    /// The declaration line and everything after it; none when nothing is declared.
    fn split(&self) -> Option<(&'a str, impl Iterator<Item = &'a str>)> {
        let mut lines = self.text.lines().map(str::trim);
        let mut in_front_matter = false;
        for (index, line) in lines.by_ref().enumerate() {
            if index == 0 && line == "---" {
                in_front_matter = true;
                continue;
            }
            if in_front_matter {
                in_front_matter = line != "---";
                continue;
            }
            if line.is_empty() || line.starts_with("%%") {
                continue;
            }
            return Some((line, lines));
        }
        None
    }

    fn statements(&self) -> impl Iterator<Item = &'a str> {
        self.split().into_iter().flat_map(|(_, rest)| rest).filter(|line| !line.is_empty() && !line.starts_with("%%"))
    }
}

fn is_message(line: &str) -> bool {
    let first_word = line.split(|c: char| c.is_whitespace() || c == ':').next().unwrap_or("").to_ascii_lowercase();
    if SEQUENCE_STATEMENTS.contains(&first_word.as_str()) {
        return false;
    }
    let before_text = line.split(':').next().unwrap_or(line);
    SEQUENCE_ARROWS.iter().any(|arrow| before_text.contains(arrow))
}

fn contains_token(line: &str, id: &str) -> bool {
    line.match_indices(id).any(|(at, _)| {
        let before = line[..at].chars().next_back();
        let after = line[at + id.len()..].chars().next();
        !before.is_some_and(is_identifier_char) && !after.is_some_and(is_identifier_char)
    })
}

fn is_identifier_char(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

#[cfg(test)]
#[path = "../../tests/unit/mermaid/mermaid_text_tests.rs"]
mod tests;
