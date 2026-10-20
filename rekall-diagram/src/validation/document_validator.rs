use crate::document::{DiagramDocument, DiagramKind, DiagramSheet, SourceLocation, Walkthrough, WalkthroughMessage};
use crate::mermaid::{ElementKey, MermaidText};

use super::{DocumentViolation, DocumentViolations};

pub const MAX_SHEETS: usize = 8;
const MAX_TITLE_CHARACTERS: usize = 120;
const MAX_MERMAID_CHARACTERS: usize = 100_000;
const MAX_LINKS_PER_SHEET: usize = 400;
const MAX_SOURCES_PER_LINK: usize = 8;
const MAX_EXPLAIN_CHARACTERS: usize = 1_000;
const MAX_VALUE_CHARACTERS: usize = 2_000;

/// Checks a parsed document against what the console relies on: every sheet declares a Mermaid
/// type, every link names something its sheet draws and a sane span of code, and a walkthrough
/// speaks of messages its sequence sends and opens only other sequence sheets.
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
        let kinds: Vec<Option<DiagramKind>> = document.sheets.iter().map(DiagramSheet::kind).collect();
        for (index, sheet) in document.sheets.iter().enumerate() {
            let at = format!("sheets[{index}]");
            self.check_sheet(&at, sheet);
            if let Some(walkthrough) = &sheet.walkthrough {
                self.check_walkthrough(&format!("{at}.walkthrough"), index, sheet, walkthrough, &kinds);
            }
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
            Ok(ElementKey::Message(number)) => self.check_message_number(path, text, number),
            Ok(ElementKey::Element(id)) => {
                if !text.mentions(id) {
                    self.refuse(path, &format!("the Mermaid draws nothing with the id \"{id}\""));
                }
            }
        }
    }

    fn check_message_number(&mut self, path: &str, text: &MermaidText, number: u32) {
        let sent = text.message_count();
        if number > sent {
            self.refuse(path, &format!("there is no message #{number}: the sequence sends {sent}"));
        }
    }

    /// `kinds` holds every sheet's type by index, which is what `opens` points into.
    fn check_walkthrough(&mut self, at: &str, index: usize, sheet: &DiagramSheet, walkthrough: &Walkthrough, kinds: &[Option<DiagramKind>]) {
        match sheet.kind() {
            None => return,
            Some(DiagramKind::Sequence) => {}
            Some(kind) => return self.refuse(at, &format!("only a sequence plays as a walkthrough; this sheet is a {kind}")),
        }
        let text = MermaidText::new(&sheet.mermaid);
        for (position, id) in walkthrough.external.iter().enumerate() {
            if !text.mentions(id) {
                self.refuse(&format!("{at}.external[{position}]"), &format!("the sequence has no participant \"{id}\""));
            }
        }
        if walkthrough.messages.len() > MAX_LINKS_PER_SHEET {
            self.refuse(&format!("{at}.messages"), &format!("at most {MAX_LINKS_PER_SHEET} messages, found {}", walkthrough.messages.len()));
        }
        for (key, message) in &walkthrough.messages {
            let path = format!("{at}.messages[{key:?}]");
            match ElementKey::parse(key) {
                Ok(ElementKey::Message(number)) => self.check_message_number(&path, &text, number),
                Ok(ElementKey::Element(_)) => self.refuse(&path, "a walkthrough addresses messages by number, like `#3`"),
                Err(reason) => self.refuse(&path, &reason),
            }
            self.check_walkthrough_message(&path, index, message, kinds);
        }
    }

    fn check_walkthrough_message(&mut self, at: &str, index: usize, message: &WalkthroughMessage, kinds: &[Option<DiagramKind>]) {
        if message.is_empty() {
            return self.refuse(at, "give it an `explain`, `input`, `output` or `opens`, or leave the message out");
        }
        if let Some(explain) = &message.explain {
            self.check_length(&format!("{at}.explain"), explain, MAX_EXPLAIN_CHARACTERS);
        }
        for (field, value) in [("input", &message.input), ("output", &message.output)] {
            if let Some(value) = value {
                self.check_length(&format!("{at}.{field}"), value, MAX_VALUE_CHARACTERS);
            }
        }
        let Some(target) = message.opens else { return };
        let path = format!("{at}.opens");
        match kinds.get(target) {
            _ if target == index => self.refuse(&path, "a sheet cannot open itself"),
            None => self.refuse(&path, &format!("there is no sheet {target}: sheets are counted from 0 and the diagram has {}", kinds.len())),
            Some(Some(DiagramKind::Sequence)) => {}
            Some(_) => self.refuse(&path, &format!("sheet {target} is not a sequence, so it cannot be walked through")),
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
