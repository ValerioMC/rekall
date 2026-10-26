use std::collections::HashSet;

use rekall_common::{jstr, Id};

use super::ProjectComponentView;
use crate::connection::ConnectionView;

/// How much of a component's description travels with its first reference.
pub const COMPONENT_REFERENCE_MAX: usize = 1_000;

/// What opens a reference to one of the project's connection bindings: `[[connection:label]]`.
const CONNECTION_REFERENCE_PREFIX: &str = "connection:";

/// Expands `[[Name]]` and `[[connection:label]]` in the text a session reads, from one project's
/// components and connection bindings.
///
/// The first reference to a component carries its description, later ones only name and folder.
/// Code spans and fenced blocks are left alone. A name that matches nothing is flagged, never dropped.
pub struct ComponentReferences<'a> {
    components: Option<&'a [ProjectComponentView]>,
    connections: &'a [ConnectionView],
    described: HashSet<Id>,
}

impl<'a> ComponentReferences<'a> {
    /// `None` is a record with no project behind it: its text passes through untouched.
    pub fn new(components: Option<&'a [ProjectComponentView]>) -> Self {
        Self { components, connections: &[], described: HashSet::new() }
    }

    pub fn with_connections(self, connections: &'a [ConnectionView]) -> Self {
        Self { connections, ..self }
    }

    pub fn expand(&mut self, text: &str) -> String {
        if self.components.is_none() || !text.contains("[[") {
            return text.to_string();
        }
        let mut out = String::with_capacity(text.len());
        let mut fence: Option<Fence> = None;
        for line in text.split_inclusive('\n') {
            match (fence, Fence::opening(line)) {
                (None, Some(opening)) => {
                    fence = Some(opening);
                    out.push_str(line);
                }
                (Some(open), marker) => {
                    if marker.is_some_and(|closing| closing.closes(open)) {
                        fence = None;
                    }
                    out.push_str(line);
                }
                (None, None) => self.expand_line(line, &mut out),
            }
        }
        out
    }

    fn expand_line(&mut self, line: &str, out: &mut String) {
        let mut at = 0;
        while at < line.len() {
            let next = line[at..].find(['`', '[']).map_or(line.len(), |offset| at + offset);
            out.push_str(&line[at..next]);
            at = next;
            if at == line.len() {
                break;
            }
            if line.as_bytes()[at] == b'`' {
                let end = code_span_end(line, at);
                out.push_str(&line[at..end]);
                at = end;
            } else if let Some((name, end)) = reference_at(line, at) {
                out.push_str(&self.expansion_of(name));
                at = end;
            } else {
                out.push('[');
                at += 1;
            }
        }
    }

    fn expansion_of(&mut self, name: &str) -> String {
        match jstr::strip(name).strip_prefix(CONNECTION_REFERENCE_PREFIX) {
            Some(label) => self.connection_expansion_of(name, label),
            None => self.component_expansion_of(name),
        }
    }

    /// The binding by its label, pointing at its `<connections>` line, which says how to reach it.
    fn connection_expansion_of(&self, name: &str, label: &str) -> String {
        let wanted = jstr::lower(jstr::strip(label));
        let Some(connection) = self.connections.iter().find(|c| jstr::lower(&c.label) == wanted) else {
            return format!("[[{name}]] (connection not found in this project)");
        };
        format!("**{}** (connection \"{}\", reached as its line in `<connections>` says)", connection.label, connection.title)
    }

    fn component_expansion_of(&mut self, name: &str) -> String {
        let wanted = jstr::lower(jstr::strip(name));
        let found = self.components.and_then(|all| all.iter().find(|c| jstr::lower(jstr::strip(&c.name)) == wanted));
        let Some(component) = found else {
            return format!("[[{name}]] (component not found in this project)");
        };
        let place = if jstr::is_blank(&component.folder) {
            "component".to_string()
        } else {
            format!("component, folder `{}`", component.folder)
        };
        let head = format!("**{}** ({place})", component.name);
        if !self.described.insert(component.id) || jstr::is_blank(&component.description) {
            return head;
        }
        format!("{head}: {}", jstr::prefix(jstr::strip(&component.description), COMPONENT_REFERENCE_MAX))
    }
}

/// A line that opens or closes a fenced code block: three or more backticks or tildes.
#[derive(Clone, Copy)]
struct Fence {
    marker: u8,
    length: usize,
}

impl Fence {
    fn opening(line: &str) -> Option<Self> {
        let trimmed = line.trim_start_matches(' ');
        let marker = *trimmed.as_bytes().first().filter(|b| matches!(b, b'`' | b'~'))?;
        let length = trimmed.bytes().take_while(|b| *b == marker).count();
        (length >= 3 && line.len() - trimmed.len() < 4).then_some(Self { marker, length })
    }

    fn closes(self, open: Fence) -> bool {
        self.marker == open.marker && self.length >= open.length
    }
}

/// The name and end of a `[[name]]` starting at `start`, on one line. A blank name, a nested
/// `[` right after the opening, or no closing `]]` is not a reference.
fn reference_at(line: &str, start: usize) -> Option<(&str, usize)> {
    let rest = line[start..].strip_prefix("[[")?;
    let close = rest.find("]]")?;
    let name = &rest[..close];
    (!jstr::is_blank(name) && !name.starts_with('[')).then_some((name, start + 2 + close + 2))
}

/// Where the code span opened at `start` ends. An opening run with no matching closing run
/// is plain text: only the run itself is skipped.
fn code_span_end(line: &str, start: usize) -> usize {
    let run = line[start..].bytes().take_while(|b| *b == b'`').count();
    let mut at = start + run;
    while let Some(offset) = line[at..].find('`') {
        let found = at + offset;
        let closing = line[found..].bytes().take_while(|b| *b == b'`').count();
        if closing == run {
            return found + closing;
        }
        at = found + closing;
    }
    start + run
}

#[cfg(test)]
#[path = "../../tests/unit/component/component_references_tests.rs"]
mod tests;
