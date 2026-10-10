use crate::document::{DiagramDocument, DiagramKind, DiagramSheet, SourceLocation};
use crate::mermaid::{ElementKey, MermaidText};

use super::{DocumentViolation, DocumentViolations};

pub const MAX_SHEETS: usize = 8;
const MAX_TITLE_CHARACTERS: usize = 120;
const MAX_MERMAID_CHARACTERS: usize = 100_000;
const MAX_LINKS_PER_SHEET: usize = 400;
const MAX_SOURCES_PER_LINK: usize = 8;

/// Checks a parsed document against what the console relies on: every sheet declares a Mermaid
/// type, and every link names something its sheet draws and a sane span of code.
pub struct DocumentValidator {
    violations: Vec<DocumentViolation>,
}

impl DocumentValidator {
    pub fn validate(document: &DiagramDocument) -> Result<(), DocumentViolations> {
        let mut validator = Self { violations: Vec::new() };
        validator.check(document);
        DocumentViolations::into_result(validator.violations)
    }

    fn check(&mut self, document: &DiagramDocument) {
        if document.sheets.is_empty() {
            self.refuse("sheets", "a diagram needs at least one sheet");
        }
        if document.sheets.len() > MAX_SHEETS {
            self.refuse("sheets", &format!("at most {MAX_SHEETS} sheets, found {}", document.sheets.len()));
        }
        for (index, sheet) in document.sheets.iter().enumerate() {
            self.check_sheet(&format!("sheets[{index}]"), sheet);
        }
    }

    fn check_sheet(&mut self, at: &str, sheet: &DiagramSheet) {
        if let Some(title) = &sheet.title {
            self.check_length(&format!("{at}.title"), title, MAX_TITLE_CHARACTERS);
        }
        let Some(kind) = self.check_mermaid(&format!("{at}.mermaid"), &sheet.mermaid) else { return };
        if sheet.links.is_empty() {
            return;
        }
        if !kind.links_elements() {
            return self.refuse(
                &format!("{at}.links"),
                &format!("a {kind} sheet is drawn by Mermaid itself, so its elements cannot open code; leave `links` out"),
            );
        }
        if sheet.links.len() > MAX_LINKS_PER_SHEET {
            self.refuse(&format!("{at}.links"), &format!("at most {MAX_LINKS_PER_SHEET} links, found {}", sheet.links.len()));
        }
        let text = MermaidText::new(&sheet.mermaid);
        for (key, sources) in &sheet.links {
            let path = format!("{at}.links[{key:?}]");
            self.check_target(&path, kind, &text, key);
            self.check_sources(&path, sources);
        }
    }

    /// The Mermaid has to be there and declare a type; that type is what the rest is held to.
    fn check_mermaid(&mut self, path: &str, mermaid: &str) -> Option<DiagramKind> {
        if mermaid.trim().is_empty() {
            self.refuse(path, "must not be blank");
            return None;
        }
        self.check_length(path, mermaid, MAX_MERMAID_CHARACTERS);
        let kind = MermaidText::new(mermaid).declared_kind();
        if kind.is_none() {
            self.refuse(
                path,
                "the first line has to declare a Mermaid diagram type, such as `sequenceDiagram`, `flowchart LR`, \
                 `stateDiagram-v2`, `classDiagram` or `erDiagram`",
            );
        }
        kind
    }

    fn check_target(&mut self, path: &str, kind: DiagramKind, text: &MermaidText, key: &str) {
        if key.is_empty() {
            return self.refuse(path, "the key must not be blank");
        }
        match ElementKey::parse(key) {
            Err(reason) => self.refuse(path, &reason),
            Ok(ElementKey::Message(_)) if kind != DiagramKind::Sequence => {
                self.refuse(path, "message numbers (`#n`) only address the messages of a sequence")
            }
            Ok(ElementKey::Message(number)) => {
                let sent = text.message_count();
                if number > sent {
                    self.refuse(path, &format!("there is no message #{number}: the sequence sends {sent}"));
                }
            }
            Ok(ElementKey::Element(id)) => {
                if !text.mentions(id) {
                    self.refuse(path, &format!("the Mermaid draws nothing with the id \"{id}\""));
                }
            }
        }
    }

    fn check_sources(&mut self, path: &str, sources: &[SourceLocation]) {
        if sources.is_empty() {
            self.refuse(path, "list at least one source, or leave the element out of `links`");
        }
        if sources.len() > MAX_SOURCES_PER_LINK {
            self.refuse(path, &format!("at most {MAX_SOURCES_PER_LINK} sources, found {}", sources.len()));
        }
        for (index, source) in sources.iter().enumerate() {
            self.check_source(&format!("{path}[{index}]"), source);
        }
    }

    fn check_source(&mut self, at: &str, source: &SourceLocation) {
        if source.file.trim().is_empty() {
            self.refuse(&format!("{at}.file"), "must not be blank");
        }
        match (source.start_line, source.end_line) {
            (Some(0), _) => self.refuse(&format!("{at}.startLine"), "lines are counted from 1"),
            (None, Some(_)) => self.refuse(&format!("{at}.endLine"), "an end line needs a start line"),
            (Some(start), Some(end)) if end < start => self.refuse(&format!("{at}.endLine"), "ends before it starts"),
            _ => {}
        }
    }

    fn check_length(&mut self, path: &str, text: &str, max: usize) {
        if text.chars().count() > max {
            self.refuse(path, &format!("at most {max} characters"));
        }
    }

    fn refuse(&mut self, path: &str, message: &str) {
        self.violations.push(DocumentViolation::new(path, message));
    }
}

#[cfg(test)]
#[path = "../../tests/unit/validation/document_validator_tests.rs"]
mod tests;
